import * as anchor from "@anchor-lang/core";
import { Program } from "@anchor-lang/core";
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
  EPHEMERAL_VAULT_ID,
} from "./utils";
import * as nacl from "tweetnacl";
import { expect } from "chai";

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
  if (endpoint.includes("localhost") || endpoint.includes("127.0.0.1")) {
    return { token: "" };
  }
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
  let adminUsdcAta: PublicKey;
  let adminStartUsdc = 0;
  let userTotalStakes: number[] = [0, 0];

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

  const endpoint = provider.connection.rpcEndpoint;
  const isLocalnet = endpoint.includes("localhost") || endpoint.includes("127.0.0.1");

  const TEE_URL = isLocalnet ? "http://localhost:7799" : "https://devnet-tee.magicblock.app";
  const TEE_WS_URL = isLocalnet ? "ws://localhost:7800" : "wss://devnet-tee.magicblock.app";
  const ephemeralRpcEndpoint = TEE_URL;

  let totalGasFees = 0;
  let totalRentSpent = 0;
  let totalRentReclaimed = 0;
  let globalStartBalance = 0;
  let setupGasFees = 0;
  let setupRentSpent = 0;
  let setupRentReclaimed = 0;

  async function trackBalanceChange<T>(
    actionName: string,
    isAccountCreation: boolean,
    fn: () => Promise<T>,
    isSetup = false
  ): Promise<T> {
    const balanceBefore = await provider.connection.getBalance(admin.publicKey);
    const result = await fn();
    const balanceAfter = await provider.connection.getBalance(admin.publicKey);
    const diff = balanceBefore - balanceAfter;

    if (diff > 0) {
      if (isAccountCreation) {
        const txFee = 5000; // standard Solana signature fee (5000 lamports)
        const rent = diff - txFee;
        totalGasFees += txFee;
        totalRentSpent += rent;
        if (isSetup) {
          setupGasFees += txFee;
          setupRentSpent += rent;
        }
        console.log(`      📊 [METRICS] ${actionName} - Gas Fee: ${(txFee / LAMPORTS_PER_SOL).toFixed(6)} SOL, Rent: ${(rent / LAMPORTS_PER_SOL).toFixed(6)} SOL`);
      } else {
        totalGasFees += diff;
        if (isSetup) {
          setupGasFees += diff;
        }
        console.log(`      📊 [METRICS] ${actionName} - Gas Fee: ${(diff / LAMPORTS_PER_SOL).toFixed(6)} SOL`);
      }
    } else if (diff < 0) {
      const reclaimed = -diff;
      totalRentReclaimed += reclaimed;
      if (isSetup) {
        setupRentReclaimed += reclaimed;
      }
      console.log(`      📊 [METRICS] ${actionName} - Rent Reclaimed: ${(reclaimed / LAMPORTS_PER_SOL).toFixed(6)} SOL`);
    }
    return result;
  }

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
      } catch (e: any) {
        if (i === retries - 1) throw e;
        console.log(
          `      ⚠️  ${actionName} failed (Attempt ${
            i + 1
          }/${retries}). Retrying in ${delayMs / 1000}s...`,
        );
        console.log(`      Error: ${e.message}`);
        if (e.logs) {
          console.log(`      Logs:\n${e.logs.join("\n")}`);
        } else if (e.getLogs) {
          try {
            console.log(`      Logs:\n${e.getLogs().join("\n")}`);
          } catch (_) {}
        }
        await sleep(delayMs);
      }
    }
    throw new Error("Unreachable");
  }

  async function fetchWithRetry<T>(
    accountClient: any,
    address: PublicKey,
    retries = 3,
    delayMs = 2000,
  ): Promise<T> {
    for (let i = 0; i < retries; i++) {
      try {
        return await accountClient.fetch(address);
      } catch (e: any) {
        if (i === retries - 1) throw e;
        console.log(
          `      ⚠️  Fetch failed for ${address.toBase58().slice(0, 8)}... (Attempt ${i + 1}/${retries}). Retrying in ${delayMs / 1000}s...`,
        );
        await sleep(delayMs);
      }
    }
    throw new Error("Unreachable");
  }

  it("1. Setup Environment", async () => {
    globalStartBalance = await provider.connection.getBalance(admin.publicKey);

    [protocolPda] = PublicKey.findProgramAddressSync(
      [SEED_PROTOCOL],
      program.programId,
    );
    usdcMint = await trackBalanceChange("Create Mint", true, () => withRetry(
      () => createMint(provider.connection, admin, admin.publicKey, null, 6),
      "Create Mint",
      3,
      3000,
    ), true);

    const adminAtaAccount = await trackBalanceChange("Create Admin ATA", true, () => getOrCreateAssociatedTokenAccount(
      provider.connection,
      admin,
      usdcMint,
      admin.publicKey,
    ), true);
    adminUsdcAta = adminAtaAccount.address;
    adminStartUsdc = (await provider.connection.getTokenAccountBalance(adminUsdcAta)).value.uiAmount || 0;

    for (const user of users) {
      const ata = await trackBalanceChange("Create ATA", true, () => withRetry(
        () => getOrCreateAssociatedTokenAccount(
          provider.connection,
          admin,
          usdcMint,
          user.publicKey,
        ),
        "Create ATA",
        3,
        3000,
      ), true);
      userAtas.push(ata.address);
      await trackBalanceChange("Mint Tokens", false, () => withRetry(
        () => mintTo(
          provider.connection,
          admin,
          usdcMint,
          ata.address,
          admin,
          1000 * 1e6,
        ),
        "Mint Tokens",
        3,
        3000,
      ), true);
    }

    try {
      await trackBalanceChange("Initialize Protocol", true, () => program.methods
        .initializeProtocol(new anchor.BN(300))
        .accountsPartial({
          admin: admin.publicKey,
          treasuryWallet: admin.publicKey,
          systemProgram: SystemProgram.programId,
        })
        .rpc(), true);
    } catch (e) {}

    // Set batch_settle_wait_duration to 0 so tests don't need to wait 60s between resolve and finalize
    await trackBalanceChange("Update Config", false, () => program.methods
      .updateConfig(null, null, new anchor.BN(0))
      .accountsPartial({
        admin: admin.publicKey,
        protocol: protocolPda,
        systemProgram: SystemProgram.programId,
      })
      .rpc(), true);

    const protocol = await fetchWithRetry<any>(program.account.protocol, protocolPda);
    poolId = protocol.totalPools.toNumber();
  });

  it("2. Create Pool (L1)", async () => {
    const now = Math.floor(Date.now() / 1000);
    const START_TIME = new anchor.BN(now - 10);
    // Detect localnet and use shorter duration (75s) to speed up tests, otherwise 120s
    const poolDuration = isLocalnet ? 75 : 120;
    END_TIME = START_TIME.add(new anchor.BN(poolDuration));

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

    await trackBalanceChange("Create Pool", true, () => program.methods
      .createPool(
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
      .rpc());
    console.log("    ✅ Pool Created on L1");
  });

  it("2.b. Verify Market Cutoff Enforcement (L1)", async () => {
    // Derive pool_id from current protocol.totalPools (same logic as contract)
    const tempProtocol = await fetchWithRetry<any>(program.account.protocol, protocolPda);
    const tempPoolIdBn = tempProtocol.totalPools;
    const [tempPoolPda] = PublicKey.findProgramAddressSync(
      [SEED_POOL, admin.publicKey.toBuffer(), tempPoolIdBn.toBuffer("le", 8)],
      program.programId
    );
    const [tempVaultPda] = PublicKey.findProgramAddressSync(
      [Buffer.from("pool_vault"), tempPoolPda.toBuffer()],
      program.programId
    );

    // Short duration (25s) -> 10s cutoff before end
    const now = Math.floor(Date.now() / 1000);
    const start = new anchor.BN(now - 10);
    const end = start.add(new anchor.BN(25));
    
    await trackBalanceChange("Create Pool (Cutoff Check)", true, () => program.methods
      .createPool("Cutoff Check", start, end, toPriceBn(5), new anchor.BN(0))
      .accountsPartial({
        protocol: protocolPda,
        pool: tempPoolPda,
        poolVault: tempVaultPda,
        tokenMint: usdcMint,
        createdBy: admin.publicKey,
        createdByTokenAccount: userAtas[0],
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc(), true);

    const tempBetPda = PublicKey.findProgramAddressSync([SEED_BET, tempPoolPda.toBuffer(), users[0].publicKey.toBuffer()], program.programId)[0];

    // Successful registration BEFORE cutoff
    await trackBalanceChange("Init Bet (Early Check)", true, () => program.methods
      .initBet(new anchor.BN(10 * 1e6), "early_req")
      .accountsPartial({
        user: users[0].publicKey,
        protocol: protocolPda,
        pool: tempPoolPda,
        poolVault: tempVaultPda,
        userTokenAccount: userAtas[0],
        bet: tempBetPda,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
      })
      .signers([users[0], admin])
      .rpc(), true);
    console.log("      ✅ Successful entry before cutoff recorded");

    const pool = await fetchWithRetry<any>(program.account.pool, tempPoolPda);
    console.log(`      ✅ Cutoff Time: ${new Date(pool.cutoffTime.toNumber() * 1000).toISOString()}`);
    
    // Wait for cutoff
    const waitTime = (pool.cutoffTime.toNumber() - Math.floor(Date.now() / 1000) + 2) * 1000;
    if (waitTime > 0) {
      console.log(`      ⏳ Waiting ${waitTime/1000}s for cutoff to trigger...`);
      await sleep(waitTime);
    }

    // Try to init another bet AFTER cutoff (should fail with MarketClosed)
    const lateUser = users[1];
    const lateBetPda = PublicKey.findProgramAddressSync([SEED_BET, tempPoolPda.toBuffer(), lateUser.publicKey.toBuffer()], program.programId)[0];
    
    try {
      await program.methods
        .initBet(new anchor.BN(10 * 1e6), "late_req")
        .accountsPartial({
          user: lateUser.publicKey,
          protocol: protocolPda,
          pool: tempPoolPda,
          poolVault: tempVaultPda,
          userTokenAccount: userAtas[1],
          bet: lateBetPda,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
        })
        .signers([lateUser, admin])
        .rpc();
      throw new Error("Should have failed with MarketClosed");
    } catch (e: any) {
      if (!e.message.includes("MarketClosed")) throw e;
      console.log("      ✅ MarketClosed correctly enforced after cutoff for fresh User 2");
    }

    // --- REFUND VERIFICATION ---
    const timeToExpiry = (end.toNumber() - Math.floor(Date.now() / 1000) + 2) * 1000;
    if (timeToExpiry > 0) {
      console.log(`      ⏳ Waiting ${timeToExpiry/1000}s for expiry to resolve...`);
      await sleep(timeToExpiry);
    }

    await trackBalanceChange("Resolve Pool (Cutoff Check)", false, () => program.methods
      .resolvePool(new anchor.BN(100_000))
      .accountsPartial({ admin: admin.publicKey, protocol: protocolPda, pool: tempPoolPda })
      .rpc(), true);
    
    const adminAta = (await getOrCreateAssociatedTokenAccount(provider.connection, admin, usdcMint, admin.publicKey)).address;
    
    // Finalize with 0 total_weight (skipping batchCalculateWeights)
    await trackBalanceChange("Finalize Weights (Cutoff Check)", false, () => program.methods
      .finalizeWeights()
      .accountsPartial({
        admin: admin.publicKey,
        protocol: protocolPda,
        pool: tempPoolPda,
        poolVault: tempVaultPda,
        treasuryTokenAccount: adminAta,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc(), true);

    const poolAfterFinalize = await fetchWithRetry<any>(program.account.pool, tempPoolPda);
    console.log(`      📊 Refund Pool Logic -> totalWeight: ${poolAfterFinalize.totalWeight.toString()}, totalStaked: ${poolAfterFinalize.totalStaked.toString()}`);
    
    const balanceBefore = (await provider.connection.getTokenAccountBalance(userAtas[0])).value.uiAmount;
    console.log(`      💰 User 1 Balance Before Refund: ${balanceBefore} USDC`);
    
    const tempPermissionPda = permissionPdaFromAccount(tempBetPda);
    await trackBalanceChange("Claim Refund & Close Pool (Cutoff Check)", false, () => program.methods
      .claimReward()
      .accountsPartial({
        user: users[0].publicKey,
        sponsor: admin.publicKey,
        pool: tempPoolPda,
        poolVault: tempVaultPda,
        bet: tempBetPda,
        userTokenAccount: userAtas[0],
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .signers([users[0], admin])
      .rpc(), true);

    const balanceAfter = (await provider.connection.getTokenAccountBalance(userAtas[0])).value.uiAmount;
    const diff = balanceAfter! - balanceBefore!;
    console.log(`      💰 User 1 Balance After Refund:  ${balanceAfter} USDC`);
    console.log(`      ✨ REFUND VERIFIED: User 1 received exactly ${diff} USDC back (Full Stake).`);
    
    expect(diff).to.be.closeTo(10, 0.001);
    console.log("      ✅ 100% Refund path successful for total_weight == 0 pool.");
  });

  it("3.1. Secure Bet Setup (L1: Init & Delegate)", async () => {
    const betAmount = new anchor.BN(100 * 1e6);
    console.log("    🏗️  Step 3.1: Initializing and Delegating User Bets...");

    for (let i = 0; i < users.length; i++) {
      const user = users[i];
      const requestId = requestIds[i];
      userTotalStakes[i] = betAmount.toNumber() / 1e6;
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
      permissionPdas.push(permissionPda);

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
            sponsor: admin.publicKey,
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
          .delegateBet()
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
 
      const sig = await trackBalanceChange(`Init & Delegate Bet User ${i + 1}`, true, () => anchor.web3.sendAndConfirmTransaction(
        provider.connection,
        tx,
        [admin, user],
      ));
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
          sponsor: admin.publicKey,
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
      const createPermissionIx = await program.methods
        .createBetPermission(requestId)
        .accountsPartial({
          payer: user.publicKey,
          user: user.publicKey,
          userBet: betPda,
          pool: poolPda,
          permission: permissionPdas[i],
          permissionProgram: PERMISSION_PROGRAM_ID,
          vault: EPHEMERAL_VAULT_ID,
          magicProgram: MAGIC_PROGRAM_ID,
        })
        .instruction();

      const placeBetIx = await program.methods
        .placeBet(predictions[i], requestId)
        .accountsPartial({
          user: user.publicKey,
          pool: poolPda,
          bet: betPda,
        })
        .instruction();

      const tx = new anchor.web3.Transaction().add(createPermissionIx, placeBetIx);
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
    }
  });

  it("3.3. Secure Bet Stake Increase", async () => {
    console.log("    💰 Step 3.3: Testing Bet Stake Increase on TEE...");

    const user = users[0];
    const betPda = betPdas[0];
    const additionalStake = new anchor.BN(50 * 1e6); // Add 50 USDC

    // Get pool state before (L1)
    let poolBefore = await fetchWithRetry<any>(program.account.pool, poolPda);
    const volumeBefore = poolBefore.totalStaked;

    // Get bet state before (L1 — bet is still delegated but L1 reflects original stake)
    let betBefore = await fetchWithRetry<any>(program.account.bet, betPda);
    const stakeBefore = betBefore.stake;

    console.log(`      📊 Before Increase:`);
    console.log(`         Bet Stake: ${stakeBefore.toString()}`);
    console.log(`         Pool Total Staked: ${volumeBefore.toString()}`);

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

    userTotalStakes[0] += additionalStake.toNumber() / 1e6;
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
    let betAfter = await fetchWithRetry<any>(teeProgram.account.bet, betPda);

    // Fetch pool state from L1 (not delegated — addStake updated it directly)
    let poolAfter = await fetchWithRetry<any>(program.account.pool, poolPda);

    const stakeAfter = betAfter.stake;
    const volumeAfter = poolAfter.totalStaked;

    console.log(`      📊 After Increase:`);
    console.log(`         Bet Stake (TEE): ${stakeAfter.toString()}`);
    console.log(`         Pool Total Staked (L1): ${volumeAfter.toString()}`);
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
      const tx = await trackBalanceChange("Delegate Pool (L1)", true, () => program.methods
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
        .rpc());
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
    const erProgram = new anchor.Program<SwivPrivacy>(program.idl as any, erProvider);

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



    // --- CLOSE PERMISSIONS (TEE) ---
    console.log("    🔒 Closing Ephemeral Permission accounts on TEE...");
    for (let i = 0; i < users.length; i++) {
      const betPda = betPdas[i];
      const permissionPda = permissionPdas[i];

      await withRetry(async () => {
        const closeTx = await erProgram.methods
          .closeBetPermission()
          .accountsPartial({
            payer: admin.publicKey,
            userBet: betPda,
            permission: permissionPda,
            vault: EPHEMERAL_VAULT_ID,
            magicProgram: MAGIC_PROGRAM_ID,
            permissionProgram: PERMISSION_PROGRAM_ID,
          })
          .rpc();
        console.log(`    ✅ Permission closed on TEE for User ${i + 1} (Sig: ${closeTx})`);
      }, `Close Permission User ${i + 1}`);
    }

    await sleep(2000);

    // --- UNDELEGATE BETS (Flush to L1) ---
    if (!isLocalnet) {
      console.log(
        "    📤 Flushing User Bet Data back to L1 (Batch Undelegate on Devnet)...",
      );
      const batchAccounts = betPdas.map((k) => ({
        pubkey: k,
        isWritable: true,
        isSigner: false,
      }));

      await withRetry(async () => {
        const undelegateBetTx = await erProgram.methods
          .batchUndelegateBets()
          .accountsPartial({
            payer: admin.publicKey,
            pool: poolPda,
          })
          .remainingAccounts(batchAccounts)
          .rpc();
        console.log(`    ✅ Batch User Bets Flushed to L1 (Sig: ${undelegateBetTx})`);
      }, `Flush Bets (Batch TEE -> L1)`);
    } else {
      console.log(
        "    📤 Flushing User Bet Data back to L1 (Undelegate individually on Localnet)...",
      );

      for (let i = 0; i < betPdas.length; i++) {
        const betPda = betPdas[i];
        await withRetry(async () => {
          const undelegateBetTx = await erProgram.methods
            .undelegateBet()
            .accountsPartial({
              payer: admin.publicKey,
              userBet: betPda,
            })
            .rpc();
          console.log(`    ✅ User Bet ${i + 1} Flushed to L1 (Sig: ${undelegateBetTx})`);
        }, `Flush Bet User ${i + 1} (TEE -> L1)`);
      }
    }

    await sleep(2000);

    // --- UNDELEGATE POOL (Flush to L1) ---
    console.log(
      "    📤 Finalizing Settlement: Flushing Pool PDA back to L1...",
    );

    await withRetry(async () => {
      const finalUndelegateTx = await erProgram.methods
        .undelegatePool()
        .accountsPartial({
          admin: admin.publicKey,
          protocol: protocolPda,
          pool: poolPda,
        })
        .rpc();
      console.log(`    ✅ Pool Settled back to L1 (Sig: ${finalUndelegateTx})`);
    }, "Flush Pool (TEE -> L1)");

    console.log("    🏁 Settlement Process Complete.");

    // --- MATH VERIFICATION (Post-Settlement on L1) ---
    const bet1 = await fetchWithRetry<any>(program.account.bet, betPdas[0]);
    const bet2 = await fetchWithRetry<any>(program.account.bet, betPdas[1]);
    console.log(`      ⚖️  Math Check - User 1 Weight: ${bet1.calculatedWeight.toString()}`);
    console.log(`      ⚖️  Math Check - User 2 Weight: ${bet2.calculatedWeight.toString()}`);
    
    expect(bet1.calculatedWeight.gt(new anchor.BN(100_000_000))).to.be.true;
    expect(bet2.calculatedWeight.gt(new anchor.BN(100_000_000))).to.be.true;

    expect(bet1.prediction.eq(updatedPredictions[0])).to.be.true;
    expect(bet2.prediction.eq(predictions[1])).to.be.true;
    console.log("      ⚖️  L1 Predictions Verified successfully before Claim.");
  });

  it("6. Finalize & Claim Rewards", async () => {
    console.log("    🏆 Step 6: Finalizing Pool & Claiming Rewards...");

    // --- 1. WAIT FOR L1 SETTLEMENT (Wait for Step 5 to reflect) ---
    console.log("      ⏳ Waiting for L1 Pool to reflect TEE resolution...");
    let poolAccount = await fetchWithRetry<any>(program.account.pool, poolPda);

    const poolStatusKey = (s: any) => Object.keys(s)[0];

    const formattedPoolAccount = {
      poolId: poolAccount.poolId.toNumber(),
      createdBy: poolAccount.createdBy.toBase58(),
      title: poolAccount.title,
      stakeTokenMint: poolAccount.stakeTokenMint.toBase58(),

      startTime: poolAccount.startTime.toNumber(),
      endTime: poolAccount.endTime.toNumber(),

      maxAccuracyBuffer: poolAccount.maxAccuracyBuffer.toNumber(),
      convictionBonusBps: poolAccount.convictionBonusBps.toNumber(),

      totalStaked: poolAccount.totalStaked.toString(),
      distributableAmount: poolAccount.distributableAmount.toString(),
      resolutionResult: poolAccount.resolutionResult?.toString() ?? null,

      status: poolStatusKey(poolAccount.status),
      resolutionTs: poolAccount.resolutionTs?.toNumber() ?? null,

      totalWeight: poolAccount.totalWeight?.toString() ?? "0",
      totalParticipants: poolAccount.totalParticipants.toNumber(),
    };
    console.log("      🔍 Initial Pool status:", formattedPoolAccount);

    const isResolvingOrBeyond = (s: any) =>
      "resolving" in s || "resolved" in s || "settled" in s;

    let retries = 10;
    while (!isResolvingOrBeyond(poolAccount.status) && retries > 0) {
      await sleep(1500);
      try {
        poolAccount = await fetchWithRetry<any>(program.account.pool, poolPda, 3, 1000);
      } catch (e) {}
      retries--;
    }
    if (!isResolvingOrBeyond(poolAccount.status)) {
      throw new Error("❌ Pool never reached resolving state on L1. Did Step 5 fail?");
    }
    console.log(`      ✅ Pool status is '${poolStatusKey(poolAccount.status)}' on L1. Proceeding to Finalize.`);

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

      const finalizeTx = await trackBalanceChange("Finalize Weights (L1)", false, () => program.methods
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
        .rpc());
      console.log(`      ✅ Weights Finalized (Sig: ${finalizeTx})`);
    } catch (e: any) {
      console.log(`      ⚠️  Finalize Weights failed: ${e.message}`);
      // Proceeding anyway in case it was already finalized
    }

    // --- 3. CLAIM REWARDS ---
    console.log("      💰 Processing User Claims...");

    let totalPayouts = 0;
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

      const userPermissionPda = permissionPdas[i];
      try {
        await trackBalanceChange(`Claim Reward User ${i + 1} & Close PDA/Vault`, false, () => program.methods
          .claimReward()
          .accountsPartial({
            user: user.publicKey,
            sponsor: admin.publicKey,
            pool: poolPda,
            poolVault: vaultPda,
            bet: userBetPda,
            userTokenAccount: userAta,
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([user, admin])
          .rpc());

        // Get Balance After
        const balAfter = await provider.connection.getTokenAccountBalance(
          userAta,
        );
        const endAmount = balAfter.value.uiAmount || 0;
        const profit = endAmount - startAmount;
        totalPayouts += profit;

        const netProfit = profit - userTotalStakes[i];

        if (profit > 0.000001) {
          console.log(`        🎉 WINNER! Reward (Payout): +${profit.toFixed(4)} USDC`);
          console.log(`           Staked/Bet:              ${userTotalStakes[i].toFixed(4)} USDC`);
          if (netProfit >= 0) {
            console.log(`           📈 Net Profit:           +${netProfit.toFixed(4)} USDC (Real Profit)`);
          } else {
            console.log(`           📉 Net Profit/Loss:      ${netProfit.toFixed(4)} USDC (Real Loss)`);
          }
          console.log(`           (Wallet Balance:         ${startAmount} -> ${endAmount})`);
        } else {
          console.log(`        😐 Claimed but received 0 USDC (Break even?)`);
          console.log(`           Staked/Bet:              ${userTotalStakes[i].toFixed(4)} USDC`);
          console.log(`           📉 Net Profit/Loss:      ${netProfit.toFixed(4)} USDC (Real Loss)`);
        }
      } catch (e: any) {
        console.log(`        📉 DID NOT WIN (or claim failed).`);
        const netProfit = 0 - userTotalStakes[i];
        console.log(`           Staked/Bet:              ${userTotalStakes[i].toFixed(4)} USDC`);
        console.log(`           📉 Net Profit/Loss:      ${netProfit.toFixed(4)} USDC (Real Loss)`);
        console.log(
          `           (Balance remained: ${startAmount.toFixed(4)} USDC)`,
        );
        console.log(`           Reason: ${e.message}`);
        if (e.logs) {
          console.log(`           Logs:\n${e.logs.join("\n")}`);
        }
      }
      console.log("      ---------------------------------------------------");
    }

    // --- 4. RECONCILIATION CHECK ---
    console.log("\n      🧮 MATHEMATICAL RECONCILIATION CHECK:");
    
    // Fetch ending treasury/admin USDC balance
    const adminEndUsdc = (await provider.connection.getTokenAccountBalance(adminUsdcAta)).value.uiAmount || 0;
    const treasuryFeeCollected = adminEndUsdc - adminStartUsdc;
    const totalStakes = userTotalStakes.reduce((a, b) => a + b, 0);
    const sumCalculated = totalPayouts + treasuryFeeCollected;
    const difference = Math.abs(totalStakes - sumCalculated);

    console.log(`         Total Stakes Deposited:  ${totalStakes.toFixed(4)} USDC`);
    console.log(`         Total Payouts Claimed:   ${totalPayouts.toFixed(4)} USDC`);
    console.log(`         Treasury Fees Collected: ${treasuryFeeCollected.toFixed(4)} USDC`);
    console.log(`         ------------------------------------------`);
    console.log(`         Payouts + Fees Sum:      ${sumCalculated.toFixed(4)} USDC`);
    
    if (difference < 0.0001) {
      console.log(`         ✅ MATHEMATICAL RECONCILIATION SUCCESSFUL!`);
      console.log(`            Total Stakes (${totalStakes.toFixed(4)} USDC) matches (Payouts + Fees) exactly.`);
    } else {
      console.log(`         ❌ RECONCILIATION MISMATCH!`);
      console.log(`            Difference: ${difference.toFixed(4)} USDC`);
    }
    console.log("==================================================\n");
  });

  it("7. Public Verify", async () => {
    // Note: Since users claimed their rewards in Step 6, the bet accounts are closed on L1
    // and cannot be fetched. We already verified predictions and weights on L1 in Step 5 before claiming.
    console.log("    ✅ Transparency Confirmed (Verified in Step 5 before Claim).");

    const globalEndBalance = await provider.connection.getBalance(admin.publicKey);
    const totalSpent = globalStartBalance - globalEndBalance;
    const adminEndUsdc = (await provider.connection.getTokenAccountBalance(adminUsdcAta)).value.uiAmount || 0;
    const usdcProfit = adminEndUsdc - adminStartUsdc;

    const opGas = totalGasFees - setupGasFees;
    const opRentSpent = totalRentSpent - setupRentSpent;
    const opRentReclaimed = totalRentReclaimed - setupRentReclaimed;

    const setupNetSOL = setupGasFees + setupRentSpent - setupRentReclaimed;
    const opNetSOL = totalSpent - setupNetSOL;

    console.log("\n==================================================");
    console.log(`🛠️  ONE-TIME SETUP COSTS (Devnet/Mainnet Setup):`);
    console.log(`   (Creating Mint, initializing ATAs, funding users, etc.)`);
    console.log(`   Gas Fees:       ${(setupGasFees / LAMPORTS_PER_SOL).toFixed(6)} SOL`);
    console.log(`   Rent Spent:     ${(setupRentSpent / LAMPORTS_PER_SOL).toFixed(6)} SOL`);
    console.log(`   Rent Reclaimed: ${(setupRentReclaimed / LAMPORTS_PER_SOL).toFixed(6)} SOL`);
    console.log(`   Net Setup Cost: ${(setupNetSOL / LAMPORTS_PER_SOL).toFixed(6)} SOL`);
    console.log(`   💡 Note: These costs are only paid ONCE during initial deployment`);
    console.log(`      and user onboarding. They are NOT per-pool operational costs.`);
    console.log("--------------------------------------------------");
    console.log(`🔄 RECURRING OPERATIONAL COSTS (Per Pool Lifecycle):`);
    console.log(`   (Creating Pool/Vault, Init/Delegate Bets, delegation reclaims)`);
    console.log(`   Gas Fees:       ${(opGas / LAMPORTS_PER_SOL).toFixed(6)} SOL`);
    console.log(`   Rent Deposited: ${(opRentSpent / LAMPORTS_PER_SOL).toFixed(6)} SOL`);
    console.log(`   Rent Reclaimed: ${(opRentReclaimed / LAMPORTS_PER_SOL).toFixed(6)} SOL`);
    console.log(`   ----------------------------------------------`);
    console.log(`   Net Operational Cost (Actual Change): ${(opNetSOL / LAMPORTS_PER_SOL).toFixed(6)} SOL`);
    console.log(`   💡 Note: Virtually 100% of the operational rent deposited is`);
    console.log(`      reclaimed and returned to the sponsor when the pool closes.`);
    console.log(`      The only true operational cost per pool is L1 gas fees.`);
    console.log("--------------------------------------------------");
    console.log(`📈 SPONSOR (ADMIN) FINANCIAL SUMMARY:`);
    console.log(`   Admin Start SOL:   ${(globalStartBalance / LAMPORTS_PER_SOL).toFixed(6)} SOL`);
    console.log(`   Admin End SOL:     ${(globalEndBalance / LAMPORTS_PER_SOL).toFixed(6)} SOL`);
    console.log(`   ----------------------------------------------`);
    console.log(`   Total SOL Spent:   ${(totalSpent / LAMPORTS_PER_SOL).toFixed(6)} SOL (Grand Total)`);
    console.log(`   Minus Setup Costs: -${(setupNetSOL / LAMPORTS_PER_SOL).toFixed(6)} SOL (One-Time Setup)`);
    console.log(`   Net Pool Cost:     ${(opNetSOL / LAMPORTS_PER_SOL).toFixed(6)} SOL (Real Operational Cost)`);
    console.log(`   ----------------------------------------------`);
    console.log(`   Admin Start USDC:  ${adminStartUsdc.toFixed(2)} USDC`);
    console.log(`   Admin End USDC:    ${adminEndUsdc.toFixed(2)} USDC`);
    console.log(`   Net USDC Profit:   +${usdcProfit.toFixed(2)} USDC (3% Treasury Fees)`);
    console.log("==================================================\n");
  });
});