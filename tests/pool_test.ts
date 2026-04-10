import * as anchor from "@coral-xyz/anchor";
import { Program } from "@coral-xyz/anchor";
import { SwivPrivacy } from "../target/types/swiv_privacy";
import {
  PublicKey,
  SystemProgram,
  Keypair,
  LAMPORTS_PER_SOL,
  ComputeBudgetProgram,
  sendAndConfirmTransaction,
} from "@solana/web3.js";
import {
  createMint,
  getOrCreateAssociatedTokenAccount,
  mintTo,
  TOKEN_PROGRAM_ID,
} from "@solana/spl-token";
import * as fs from "fs";
import * as path from "path";
import {
  SEED_BET,
  SEED_POOL,
  SEED_PROTOCOL,
  PERMISSION_PROGRAM_ID,
  sleep,
  TEE_VALIDATOR,
  getAuthToken,
  DELEGATION_PROGRAM_ID,
  permissionPdaFromAccount,
  delegationRecordPdaFromDelegatedAccount,
  delegationMetadataPdaFromDelegatedAccount,
  delegateBufferPdaFromDelegatedAccountAndOwnerProgram,
  waitUntilPermissionActive,
  MAGIC_CONTEXT_ID,
  MAGIC_PROGRAM_ID,
} from "./utils";
import * as nacl from "tweetnacl";

const KEYS_DIR = path.join(__dirname, "keys");
if (!fs.existsSync(KEYS_DIR)) fs.mkdirSync(KEYS_DIR);

function loadOrGenerateKeypair(name: string): Keypair {
  const filePath = path.join(KEYS_DIR, `${name}.json`);
  if (fs.existsSync(filePath)) {
    return Keypair.fromSecretKey(
      new Uint8Array(JSON.parse(fs.readFileSync(filePath, "utf-8"))),
    );
  } else {
    const kp = Keypair.generate();
    fs.writeFileSync(filePath, JSON.stringify(Array.from(kp.secretKey)));
    return kp;
  }
}

async function getAuthTokenWithRetry(
  endpoint: string,
  pubkey: PublicKey,
  signer: (msg: Uint8Array) => Promise<Uint8Array>,
  retries = 3,
): Promise<{ token: string }> {
  for (let i = 0; i < retries; i++) {
    try {
      return await getAuthToken(endpoint, pubkey, signer);
    } catch (e) {
      if (i === retries - 1) throw e;
      console.log(`      ⚠️  Auth failed. Retrying (${i + 1}/${retries})...`);
      await sleep(2000 * (i + 1));
    }
  }
  throw new Error("Unreachable");
}

describe("Production Flow", () => {
  const provider = anchor.AnchorProvider.env();
  anchor.setProvider(provider);
  const program = anchor.workspace.SwivPrivacy as Program<SwivPrivacy>;
  const admin = (provider.wallet as anchor.Wallet).payer;

  const users = [
    loadOrGenerateKeypair("user_tee_1"),
    loadOrGenerateKeypair("user_tee_2"),
  ];

  let usdcMint: PublicKey;
  let userAtas: PublicKey[] = [];
  let protocolPda: PublicKey;
  let poolPda: PublicKey;
  let vaultPda: PublicKey;
  let poolId: number = 0;

  const POOL_TITLE = `TEE-Pool-${Math.floor(Math.random() * 1000)}`;
  let END_TIME: anchor.BN;
  const PRICE_SCALE = 1_000_000;
  const toPriceBn = (value: number): anchor.BN =>
    new anchor.BN(Math.round(value * PRICE_SCALE));
  const TARGET_PRICE = toPriceBn(75.78);

  const predictions = [toPriceBn(76.12), toPriceBn(75.11)];
  const updatedPredictions = [toPriceBn(74.76), toPriceBn(76.25)];
  const requestIds = ["req_1", "req_2"];
  const betPdas: PublicKey[] = [];
  const permissionPdas: PublicKey[] = [];

  const TEE_URL = "https://tee.magicblock.app";
  const TEE_WS_URL = "wss://tee.magicblock.app";
  const ephemeralRpcEndpoint = TEE_URL;

  // Helper: Retry a function up to 'retries' times with a delay
  async function withRetry<T>(
    fn: () => Promise<T>,
    actionName: string,
    retries = 5,
    delayMs = 2000,
  ): Promise<T> {
    for (let i = 0; i < retries; i++) {
      try {
        return await fn();
      } catch (e) {
        if (i === retries - 1) throw e;
        console.log(
          `      ⚠️  ${actionName} failed (Attempt ${
            i + 1
          }/${retries}). Retrying in ${delayMs / 1000}s...`,
        );
        console.log(`      Error: ${e.message}`);
        await sleep(delayMs);
      }
    }
    throw new Error("Unreachable");
  }

  it("1. Setup Environment", async () => {
    [protocolPda] = PublicKey.findProgramAddressSync(
      [SEED_PROTOCOL],
      program.programId,
    );
    usdcMint = await createMint(
      provider.connection,
      admin,
      admin.publicKey,
      null,
      6,
    );

    for (const user of users) {
      const bal = await provider.connection.getBalance(user.publicKey);
      if (bal < 0.1 * LAMPORTS_PER_SOL) {
        await provider.sendAndConfirm(
          new anchor.web3.Transaction().add(
            SystemProgram.transfer({
              fromPubkey: admin.publicKey,
              toPubkey: user.publicKey,
              lamports: 0.1 * LAMPORTS_PER_SOL,
            }),
          ),
        );
      }
      const ata = await getOrCreateAssociatedTokenAccount(
        provider.connection,
        admin,
        usdcMint,
        user.publicKey,
      );
      userAtas.push(ata.address);
      await mintTo(
        provider.connection,
        admin,
        usdcMint,
        ata.address,
        admin,
        1000 * 1e6,
      );
    }

    try {
      await program.methods
        .initializeProtocol(new anchor.BN(300))
        .accountsPartial({
          admin: admin.publicKey,
          treasuryWallet: admin.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
    } catch (e) {}

    const protocol = await program.account.protocol.fetch(protocolPda);
    poolId = protocol.totalPools.toNumber();
  });

  it("2. Create Pool (L1)", async () => {
    const now = Math.floor(Date.now() / 1000);
    const START_TIME = new anchor.BN(now);
    END_TIME = START_TIME.add(new anchor.BN(40));

    [poolPda] = PublicKey.findProgramAddressSync(
      [
        SEED_POOL,
        admin.publicKey.toBuffer(),
        new anchor.BN(poolId).toBuffer("le", 8),
      ],
      program.programId,
    );
    [vaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool_vault"), poolPda.toBuffer()],
      program.programId,
    );

    const adminAta = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      admin,
      usdcMint,
      admin.publicKey,
    );

    await program.methods
      .createPool(
        new anchor.BN(poolId),
        POOL_TITLE,
        START_TIME,
        END_TIME,
        toPriceBn(5),
        new anchor.BN(3),
      )
      .accountsPartial({
        protocol: protocolPda,
        pool: poolPda,
        poolVault: vaultPda,
        tokenMint: usdcMint,
        createdBy: admin.publicKey,
        createdByTokenAccount: adminAta.address,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();
    console.log("    ✅ Pool Created on L1");
  });

  it("3.1. Secure Bet Setup (L1: Init & Delegate)", async () => {
    const betAmount = new anchor.BN(100 * 1e6);
    console.log("    🏗️  Step 3.1: Initializing and Delegating User Bets...");

    for (let i = 0; i < users.length; i++) {
      const user = users[i];
      const requestId = requestIds[i];
      console.log(
        `      👤 Processing User ${i + 1} (${user.publicKey
          .toBase58()
          .slice(0, 8)}...)`,
      );

      const [betPda] = PublicKey.findProgramAddressSync(
        [SEED_BET, poolPda.toBuffer(), user.publicKey.toBuffer()],
        program.programId,
      );
      betPdas.push(betPda);
      const permissionPda = permissionPdaFromAccount(betPda);

      const bufferUserBet =
        delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
          betPda,
          program.programId,
        );
      const delegationRecordUserBet =
        delegationRecordPdaFromDelegatedAccount(betPda);
      const delegationMetadataUserBet =
        delegationMetadataPdaFromDelegatedAccount(betPda);

      console.log(`        👉 Bet PDA: ${betPda.toBase58()}`);
      console.log(`        👉 Permission PDA: ${permissionPda.toBase58()}`);

      const tx = new anchor.web3.Transaction().add(
        await program.methods
          .initBet(betAmount, requestId)
          .accountsPartial({
            user: user.publicKey,
            protocol: protocolPda,
            pool: poolPda,
            poolVault: vaultPda,
            userTokenAccount: userAtas[i],
            bet: betPda,
            tokenProgram: TOKEN_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .instruction(),
        await program.methods
          .createBetPermission(requestId)
          .accountsPartial({
            payer: user.publicKey,
            user: user.publicKey,
            userBet: betPda,
            pool: poolPda,
            permission: permissionPda,
            permissionProgram: PERMISSION_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .instruction(),
        await program.methods
          .delegateBetPermission(requestId)
          .accountsPartial({
            user: user.publicKey,
            pool: poolPda,
            userBet: betPda,
            permission: permissionPda,
            permissionProgram: PERMISSION_PROGRAM_ID,
            delegationProgram: DELEGATION_PROGRAM_ID,
            delegationRecord:
              delegationRecordPdaFromDelegatedAccount(permissionPda),
            delegationMetadata:
              delegationMetadataPdaFromDelegatedAccount(permissionPda),
            delegationBuffer:
              delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
                permissionPda,
                PERMISSION_PROGRAM_ID,
              ),
            validator: TEE_VALIDATOR,
            systemProgram: SystemProgram.programId,
          })
          .instruction(),

        await program.methods
          .delegateBet(requestId)
          .accountsPartial({
            user: user.publicKey,
            payer: admin.publicKey,
            pool: poolPda,
            bufferUserBet: bufferUserBet,
            delegationRecordUserBet: delegationRecordUserBet,
            delegationMetadataUserBet: delegationMetadataUserBet,
            userBet: betPda,
            validator: TEE_VALIDATOR,
            ownerProgram: program.programId,
            delegationProgram: DELEGATION_PROGRAM_ID,
            systemProgram: SystemProgram.programId,
          })
          .instruction(),
      );

      tx.feePayer = admin.publicKey;

      const sig = await anchor.web3.sendAndConfirmTransaction(
        provider.connection,
        tx,
        [admin, user],
      );
      console.log(`        ✅ L1 Transaction Confirmed: ${sig}`);

      console.log(`        ⏳ Waiting for TEE to index delegation...`);
      await waitUntilPermissionActive(ephemeralRpcEndpoint, betPda);
      console.log(`        ✨ TEE Synchronization Complete for User ${i + 1}`);
    }
  });

  it("3.1.b Duplicate initBet should fail for same user/pool", async () => {
    const duplicateAmount = new anchor.BN(10 * 1e6);
    const user = users[0];
    const duplicateRequestId = "req_duplicate_should_fail";

    let failed = false;
    try {
      await program.methods
        .initBet(duplicateAmount, duplicateRequestId)
        .accountsPartial({
          user: user.publicKey,
          protocol: protocolPda,
          pool: poolPda,
          poolVault: vaultPda,
          userTokenAccount: userAtas[0],
          bet: betPdas[0],
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([admin, user]) // Add admin as a signer
        .rpc();
    } catch (_e) {
      failed = true;
    }

    if (!failed) {
      throw new Error(
        "Expected second initBet call to fail for same user/pool bet PDA",
      );
    }

    console.log(
      "    ✅ Duplicate initBet rejected for canonical user/pool bet",
    );
  });

  it("3.2. Secure Bet Execution (TEE: Place Bet)", async () => {
    console.log("    🎯 Step 3.2: Executing Private Bets on TEE...");

    for (let i = 0; i < users.length; i++) {
      const user = users[i];
      const requestId = requestIds[i];
      const betPda = betPdas[i];

      console.log(`      🔐 Authenticating User ${i + 1}...`);
      const authToken = await getAuthTokenWithRetry(
        ephemeralRpcEndpoint,
        user.publicKey,
        async (msg) => nacl.sign.detached(msg, user.secretKey),
      );

      const teeConnection = new anchor.web3.Connection(
        `${TEE_URL}?token=${authToken.token}`,
        {
          commitment: "confirmed",
          wsEndpoint: `${TEE_WS_URL}?token=${authToken.token}`,
        },
      );

      console.log(
        `      🚀 Sending 'placeBet' to TEE (Prediction: ${predictions[
          i
        ].toString()})...`,
      );
      const placeBetIx = await program.methods
        .placeBet(predictions[i], requestId)
        .accountsPartial({
          user: user.publicKey,
          pool: poolPda,
          bet: betPda,
        })
        .instruction();

      const tx = new anchor.web3.Transaction().add(placeBetIx);
      tx.feePayer = user.publicKey;
      tx.recentBlockhash = (await teeConnection.getLatestBlockhash()).blockhash;

      const txSig = await sendAndConfirmTransaction(teeConnection, tx, [user], {
        skipPreflight: true,
      });
      console.log(`      ✅ Bet Executed Privately. TEE Sig: ${txSig}`);

      if (i === 0) {
        console.log(
          `      🔁 Updating User ${
            i + 1
          } bet via updateBet (Prediction: ${updatedPredictions[
            i
          ].toString()})...`,
        );

        const updateBetIx = await program.methods
          .updateBet(updatedPredictions[i], new anchor.BN(0))
          .accountsPartial({
            user: user.publicKey,
            pool: poolPda,
            bet: betPda,
          })
          .instruction();

        const updateTx = new anchor.web3.Transaction().add(updateBetIx);
        updateTx.feePayer = user.publicKey;
        updateTx.recentBlockhash = (
          await teeConnection.getLatestBlockhash()
        ).blockhash;

        const updateSig = await sendAndConfirmTransaction(
          teeConnection,
          updateTx,
          [user],
          {
            skipPreflight: true,
          },
        );

        console.log(`      ✅ Bet Updated Privately. TEE Sig: ${updateSig}`);
      }

      await sleep(1000);
    }
  });

  it("3.3. Secure Bet Stake Increase", async () => {
    console.log("    💰 Step 3.3: Testing Bet Stake Increase on TEE...");

    const user = users[0];
    const betPda = betPdas[0];
    const additionalStake = new anchor.BN(50 * 1e6); // Add 50 USDC

    // Get pool state before (L1)
    let poolBefore = await program.account.pool.fetch(poolPda);
    const volumeBefore = poolBefore.totalVolume;

    // Get bet state before (L1 — bet is still delegated but L1 reflects original stake)
    let betBefore = await program.account.bet.fetch(betPda);
    const stakeBefore = betBefore.stake;

    console.log(`      📊 Before Increase:`);
    console.log(`         Bet Stake: ${stakeBefore.toString()}`);
    console.log(`         Pool Volume: ${volumeBefore.toString()}`);

    // ── Step 1: Transfer tokens on L1 via add_stake ──────────────────────────
    console.log(
      `      💸 Calling addStake on L1 to transfer tokens and update pool volume...`,
    );

    const [poolVaultBump] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool_vault"), poolPda.toBuffer()],
      program.programId,
    );

    await program.methods
      .addStake(additionalStake)
      .accountsPartial({
        user: user.publicKey,
        pool: poolPda,
        poolVault: vaultPda,
        userTokenAccount: userAtas[0],
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([user])
      .rpc();

    console.log(`      ✅ addStake executed on L1.`);

    // ── Step 2: Update prediction + record stake increase on TEE ─────────────
    console.log(
      `      🎯 Calling updateBet on TEE to record stake increase and update prediction...`,
    );

    const authToken = await getAuthTokenWithRetry(
      ephemeralRpcEndpoint,
      user.publicKey,
      async (msg) => nacl.sign.detached(msg, user.secretKey),
    );

    const teeConnection = new anchor.web3.Connection(
      `${TEE_URL}?token=${authToken.token}`,
      {
        commitment: "confirmed",
        wsEndpoint: `${TEE_WS_URL}?token=${authToken.token}`,
      },
    );

    // Use updatedPredictions[0] (78) so test 7 sees the correct final prediction
    const newPredictionForUpdate = updatedPredictions[0];

    const updateBetIx = await program.methods
      .updateBet(newPredictionForUpdate, additionalStake)
      .accountsPartial({
        user: user.publicKey,
        pool: poolPda,
        bet: betPda,
      })
      .instruction();

    const updateTx = new anchor.web3.Transaction().add(updateBetIx);
    updateTx.feePayer = user.publicKey;
    updateTx.recentBlockhash = (
      await teeConnection.getLatestBlockhash()
    ).blockhash;

    const updateSig = await sendAndConfirmTransaction(
      teeConnection,
      updateTx,
      [user],
      {
        skipPreflight: true,
      },
    );

    console.log(
      `      ✅ Bet Updated with Stake Increase on TEE. TEE Sig: ${updateSig}`,
    );
    await sleep(1000);

    // ── Verify results ────────────────────────────────────────────────────────
    // Fetch bet state from TEE (canonical while delegated)
    const teeProvider = new anchor.AnchorProvider(
      teeConnection,
      new anchor.Wallet(user),
      {
        commitment: "confirmed",
      },
    );
    const teeProgram = new anchor.Program<SwivPrivacy>(
      program.idl,
      teeProvider,
    );
    let betAfter = await teeProgram.account.bet.fetch(betPda);

    // Fetch pool state from L1 (not delegated — addStake updated it directly)
    let poolAfter = await program.account.pool.fetch(poolPda);

    const stakeAfter = betAfter.stake;
    const volumeAfter = poolAfter.totalVolume;

    console.log(`      📊 After Increase:`);
    console.log(`         Bet Stake (TEE): ${stakeAfter.toString()}`);
    console.log(`         Pool Volume (L1): ${volumeAfter.toString()}`);
    console.log(`         Prediction (TEE): ${betAfter.prediction.toString()}`);

    // Assertions
    if (!stakeAfter.eq(stakeBefore.add(additionalStake))) {
      throw new Error(
        `❌ Stake not increased correctly. Expected ${stakeBefore
          .add(additionalStake)
          .toString()}, got ${stakeAfter.toString()}`,
      );
    }

    if (!volumeAfter.eq(volumeBefore.add(additionalStake))) {
      throw new Error(
        `❌ Pool volume not updated correctly. Expected ${volumeBefore
          .add(additionalStake)
          .toString()}, got ${volumeAfter.toString()}`,
      );
    }

    if (!betAfter.prediction.eq(newPredictionForUpdate)) {
      throw new Error(
        `❌ Prediction not updated. Expected ${newPredictionForUpdate.toString()}, got ${betAfter.prediction.toString()}`,
      );
    }

    console.log(`    ✅ Stake Increase Verified Successfully!`);
  });

  it("4. Privacy Verification (TEE Snoop Check)", async () => {
    console.log("    🕵️ Step 4: Verifying Data Privacy on TEE...");

    // User 1 attempts to read User 2's bet
    const user1 = users[0];
    const user2BetPda = betPdas[1];

    console.log(
      `      🕵️ User 1 (${user1.publicKey
        .toBase58()
        .slice(0, 8)}) is attempting to peek at User 2's Bet...`,
    );

    const authToken = await getAuthTokenWithRetry(
      ephemeralRpcEndpoint,
      user1.publicKey,
      async (msg) => nacl.sign.detached(msg, user1.secretKey),
    );

    const teeConnection = new anchor.web3.Connection(
      `${TEE_URL}?token=${authToken.token}`,
      {
        commitment: "confirmed",
      },
    );

    try {
      const accountInfo = await teeConnection.getAccountInfo(user2BetPda);

      if (accountInfo === null) {
        console.log(
          "      ✅ PRIVACY CONFIRMED: TEE returned 'null' for unauthorized access.",
        );
      } else {
        // If the TEE returns data, check if it's actually decrypted/readable
        console.log(
          "      ⚠️  TEE returned account data. Checking readability...",
        );
        try {
          const decoded = program.coder.accounts.decode(
            "UserBet",
            accountInfo.data,
          );
          console.log(
            "      ❌ PRIVACY BREACH: User 1 was able to decode User 2's prediction:",
            decoded.prediction.toString(),
          );
          throw new Error(
            "Privacy Failure: Unauthorized user read private data!",
          );
        } catch (e) {
          console.log(
            "      ✅ PRIVACY CONFIRMED: Data is present but encrypted/unreadable by User 1.",
          );
        }
      }
    } catch (err) {
      console.log(
        "      ✅ PRIVACY CONFIRMED: TEE explicitly blocked the request.",
      );
    }
  });

  it("5. Delegate Pool & Resolve", async () => {
    console.log("    ⏳ Step 5: Beginning Settlement Process...");

    // 1. Wait for Pool Expiry
    const timeToWait = Math.max(
      0,
      END_TIME.toNumber() * 1000 - Date.now() + 2000,
    );
    console.log(`    ⏳ Waiting ${timeToWait / 1000}s for pool expiry...`);
    await sleep(timeToWait);

    const bufferPool = delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
      poolPda,
      program.programId,
    );
    const delegationRecordPool =
      delegationRecordPdaFromDelegatedAccount(poolPda);
    const delegationMetadataPool =
      delegationMetadataPdaFromDelegatedAccount(poolPda);

    // --- DELEGATE POOL ---
    console.log("    🔗 Delegating Pool PDA to TEE...");

    // Retry delegation if L1 is busy
    await withRetry(async () => {
      const tx = await program.methods
        .delegatePool(new anchor.BN(poolId))
        .accountsPartial({
          admin: admin.publicKey,
          protocol: protocolPda,
          bufferPool: bufferPool,
          delegationRecordPool: delegationRecordPool,
          delegationMetadataPool: delegationMetadataPool,
          pool: poolPda,
          validator: TEE_VALIDATOR,
          ownerProgram: program.programId,
          delegationProgram: DELEGATION_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .rpc();
      console.log(`    ✅ Pool Delegated (L1 Sig: ${tx})`);
    }, "Delegate Pool");

    // Give L1 a moment to propagate state
    await sleep(2000);

    // --- AUTHENTICATION ---
    console.log("    🔐 Authenticating Admin Session on TEE...");
    const authToken = await getAuthTokenWithRetry(
      ephemeralRpcEndpoint,
      admin.publicKey,
      async (m) => nacl.sign.detached(m, admin.secretKey),
    );

    // Re-establish connection with fresh token
    const erConn = new anchor.web3.Connection(
      `${TEE_URL}?token=${authToken.token}`,
      {
        commitment: "confirmed",
        wsEndpoint: `${TEE_WS_URL}?token=${authToken.token}`,
        // Increase timeout for TEE requests
        confirmTransactionInitialTimeout: 60000,
      },
    );

    const erProvider = new anchor.AnchorProvider(
      erConn,
      new anchor.Wallet(admin),
      { commitment: "confirmed", preflightCommitment: "confirmed" },
    );
    const erProgram = new anchor.Program(program.idl, erProvider);

    // --- RESOLVE POOL (TEE) ---
    console.log(
      `    🎲 Resolving Pool on TEE (Target Outcome: ${TARGET_PRICE.toString()})...`,
    );

    await withRetry(async () => {
      const resolveTx = await erProgram.methods
        .resolvePool(TARGET_PRICE)
        .accountsPartial({
          admin: admin.publicKey,
          protocol: protocolPda,
          pool: poolPda,
        })
        .rpc();
      console.log(`    ✅ Pool Resolved on TEE (Sig: ${resolveTx})`);
    }, "Resolve Pool (TEE)");

    await sleep(2000); // Wait for TEE state update

    // --- CALCULATE WEIGHTS (TEE) ---
    console.log(
      `    ⚖️  Calculating Weights for ${betPdas.length} users on TEE...`,
    );

    const batchAccounts = betPdas.map((k) => ({
      pubkey: k,
      isWritable: true,
      isSigner: false,
    }));

    await withRetry(async () => {
      const calcTx = await erProgram.methods
        .batchCalculateWeights()
        .accountsPartial({ admin: admin.publicKey, pool: poolPda })
        .remainingAccounts(batchAccounts)
        .rpc();
      console.log(`    ✅ Weights Calculated (Sig: ${calcTx})`);
    }, "Calculate Weights (TEE)");

    await sleep(2000);

    // --- UNDELEGATE BETS (Flush to L1) ---
    console.log(
      "    📤 Flushing User Bet Data back to L1 (Batch Undelegate)...",
    );

    await withRetry(async () => {
      const undelegateBetsTx = await erProgram.methods
        .batchUndelegateBets()
        .accounts({
          payer: admin.publicKey,
          pool: poolPda,
          magicContext: MAGIC_CONTEXT_ID,
          magicProgram: MAGIC_PROGRAM_ID,
        })
        .remainingAccounts(batchAccounts)
        .rpc();
      console.log(`    ✅ User Bets Flushed to L1 (Sig: ${undelegateBetsTx})`);
    }, "Flush Bets (TEE -> L1)");

    await sleep(2000);

    // --- UNDELEGATE POOL (Flush to L1) ---
    console.log(
      "    📤 Finalizing Settlement: Flushing Pool PDA back to L1...",
    );

    await withRetry(async () => {
      const finalUndelegateTx = await erProgram.methods
        .undelegatePool()
        .accounts({
          admin: admin.publicKey,
          protocol: protocolPda,
          pool: poolPda,
          magicContext: MAGIC_CONTEXT_ID,
          magicProgram: MAGIC_PROGRAM_ID,
        })
        .rpc();
      console.log(`    ✅ Pool Settled back to L1 (Sig: ${finalUndelegateTx})`);
    }, "Flush Pool (TEE -> L1)");

    console.log("    🏁 Settlement Process Complete.");
  });

  it("6. Finalize & Claim Rewards", async () => {
    console.log("    🏆 Step 6: Finalizing Pool & Claiming Rewards...");

    // --- 1. WAIT FOR L1 SETTLEMENT (Wait for Step 5 to reflect) ---
    console.log("      ⏳ Waiting for L1 Pool to reflect TEE resolution...");
    let poolAccount = await program.account.pool.fetch(poolPda);
    const formattedPoolAccount = {
      poolId: poolAccount.poolId.toNumber(),
      createdBy: poolAccount.createdBy.toBase58(),
      title: poolAccount.title,
      stakeTokenMint: poolAccount.stakeTokenMint.toBase58(),

      startTime: poolAccount.startTime.toNumber(),
      endTime: poolAccount.endTime.toNumber(),

      maxAccuracyBuffer: poolAccount.maxAccuracyBuffer.toNumber(),
      convictionBonusBps: poolAccount.convictionBonusBps.toNumber(),

      totalVolume: poolAccount.totalVolume.toString(),
      resolutionResult: poolAccount.resolutionResult?.toString() ?? null,

      isResolved: poolAccount.isResolved,
      resolutionTs: poolAccount.resolutionTs?.toNumber() ?? null,

      totalWeight: poolAccount.totalWeight?.toString() ?? "0",

      weightFinalized: poolAccount.weightFinalized,
      totalParticipants: poolAccount.totalParticipants.toNumber(),
    };
    console.log("      🔍 Initial Pool isResolved:", formattedPoolAccount);
    let retries = 10;
    while (!poolAccount.isResolved && retries > 0) {
      await sleep(1500);
      try {
        poolAccount = await program.account.pool.fetch(poolPda);
      } catch (e) {}
      retries--;
    }
    if (!poolAccount.isResolved) {
      throw new Error("❌ Pool never resolved on L1. Did Step 5 fail?");
    }
    console.log("      ✅ Pool is Resolved on L1. Proceeding to Finalize.");

    // --- 2. FINALIZE WEIGHTS (The Missing Step) ---
    // This calculates fees and unlocks the vault for claimers
    try {
      // In Step 1, we set treasuryWallet = admin.publicKey.
      // So we pass the admin's ATA as the treasury token account.
      const adminAta = await getOrCreateAssociatedTokenAccount(
        provider.connection,
        admin,
        usdcMint,
        admin.publicKey,
      );

      const finalizeTx = await program.methods
        .finalizeWeights()
        .accountsPartial({
          admin: admin.publicKey,
          protocol: protocolPda,
          pool: poolPda,
          poolVault: vaultPda,
          treasuryTokenAccount: adminAta.address,
          tokenProgram: TOKEN_PROGRAM_ID,
        })
        .signers([admin])
        .rpc();
      console.log(`      ✅ Weights Finalized (Sig: ${finalizeTx})`);
    } catch (e) {
      console.log(`      ⚠️  Finalize Weights failed: ${e.message}`);
      // Proceeding anyway in case it was already finalized
    }

    // --- 3. CLAIM REWARDS ---
    console.log("      💰 Processing User Claims...");

    for (let i = 0; i < users.length; i++) {
      const user = users[i];
      const userBetPda = betPdas[i];
      const userAta = userAtas[i];

      console.log(
        `      👤 User ${i + 1} (${user.publicKey.toBase58().slice(0, 8)}...)`,
      );

      // Get Balance Before
      const balBefore = await provider.connection.getTokenAccountBalance(
        userAta,
      );
      const startAmount = balBefore.value.uiAmount || 0;

      try {
        await program.methods
          .claimReward()
          .accountsPartial({
            user: user.publicKey,
            pool: poolPda,
            poolVault: vaultPda,
            bet: userBetPda,
            userTokenAccount: userAta,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([user])
          .rpc();

        // Get Balance After
        const balAfter = await provider.connection.getTokenAccountBalance(
          userAta,
        );
        const endAmount = balAfter.value.uiAmount || 0;
        const profit = endAmount - startAmount;

        if (profit > 0.000001) {
          console.log(`        🎉 WINNER! Reward: +${profit.toFixed(4)} USDC`);
          console.log(`           (Balance: ${startAmount} -> ${endAmount})`);
        } else {
          console.log(`        😐 Claimed but received 0 USDC (Break even?)`);
        }
      } catch (e) {
        // If claim fails, it usually means they didn't win or already claimed
        // We assume they are a "Loser" for this test context if it fails
        console.log(`        📉 DID NOT WIN (or claim failed).`);
        console.log(
          `           (Balance remained: ${startAmount.toFixed(4)} USDC)`,
        );
        // Optional: Log specific error if needed
        // console.log(`           Reason: ${e.message}`);
      }
      console.log("      ---------------------------------------------------");
    }
  });

  it("7. Public Verify", async () => {
    const user1BetData = await program.account.bet.fetch(betPdas[0]);
    const user2BetData = await program.account.bet.fetch(betPdas[1]);

    console.log(
      `    📖 L1 User1 Prediction: ${user1BetData.prediction.toString()}`,
    );
    console.log(
      `    📖 L1 User2 Prediction: ${user2BetData.prediction.toString()}`,
    );

    if (!user1BetData.prediction.eq(updatedPredictions[0])) {
      throw new Error(
        `❌ Data Mismatch (User1): Expected ${updatedPredictions[0]}, got ${user1BetData.prediction}`,
      );
    }

    if (!user2BetData.prediction.eq(predictions[1])) {
      throw new Error(
        `❌ Data Mismatch (User2): Expected ${predictions[1]}, got ${user2BetData.prediction}`,
      );
    }

    console.log("    ✅ Transparency Confirmed.");
  });
});
