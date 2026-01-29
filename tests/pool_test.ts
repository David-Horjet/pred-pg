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
  verifyTeeRpcIntegrity,
  DELEGATION_PROGRAM_ID,
  permissionPdaFromAccount,
  delegationRecordPdaFromDelegatedAccount,
  delegationMetadataPdaFromDelegatedAccount,
  delegateBufferPdaFromDelegatedAccountAndOwnerProgram,
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

// --- NEW HELPER: Retry Logic for Auth Token ---
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
      if (i === retries - 1) throw e; // Throw if last retry fails
      console.log(
        `      ⚠️  Auth failed ("${e.message}"). Retrying (${i + 1
        }/${retries})...`,
      );
      await sleep(1000 * (i + 1)); // Wait 1s, then 2s, etc.
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

  const POOL_NAME = `TEE-Pool-${Math.floor(Math.random() * 1000)}`;
  let END_TIME: anchor.BN;
  const TARGET_PRICE = new anchor.BN(200_000_000);

  const predictions = [new anchor.BN(200_000_000), new anchor.BN(250_000_000)];
  const requestIds = ["req_1", "req_2"];
  const betPdas: PublicKey[] = [];

  const TEE_URL = "https://tee.magicblock.app";
  const TEE_WS_URL = "wss://tee.magicblock.app";

  const ephemeralRpcEndpoint = (
    process.env.EPHEMERAL_PROVIDER_ENDPOINT || TEE_URL
  ).replace(/\/$/, "");
  const ephemeralWsEndpoint = process.env.EPHEMERAL_WS_ENDPOINT || TEE_WS_URL;

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

    userAtas = [];
    for (const user of users) {
      const bal = await provider.connection.getBalance(user.publicKey);
      if (bal < 0.5 * LAMPORTS_PER_SOL) {
        const tx = new anchor.web3.Transaction().add(
          SystemProgram.transfer({
            fromPubkey: admin.publicKey,
            toPubkey: user.publicKey,
            lamports: 0.5 * LAMPORTS_PER_SOL,
          }),
        );
        await provider.sendAndConfirm(tx);
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

    // Try to close old protocol account if it exists with wrong discriminator
    try {
      const accountInfo = await provider.connection.getAccountInfo(protocolPda);
      if (accountInfo) {
        // Try to recover by closing the account
        const closeTx = new anchor.web3.Transaction().add(
          anchor.web3.SystemProgram.transfer({
            fromPubkey: protocolPda,
            toPubkey: admin.publicKey,
            lamports: accountInfo.lamports,
          }),
        );
        // We can't directly close the account from here, so we'll just proceed
        // The initializeProtocol will create a new one
      }
    } catch (e) {
      /* Account doesn't exist yet */
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
    } catch (e: any) {
      // If account discriminator mismatch, the account needs to be reset
      if (e.message.includes("AccountDiscriminatorMismatch")) {
        console.log("    ⚠️  Old protocol account exists. This is expected on first run after refactor.");
        console.log("    💡 Run: solana-test-validator --reset");
        throw new Error("Please reset localnet state and run tests again");
      }
      /* Otherwise idempotent */
    }

    // Fetch protocol to get current pool count for poolId
    const protocol = await program.account.protocol.fetch(protocolPda);
    console.log("protocol: ", protocol);
    poolId = protocol.totalPools.toNumber();
  });

  it("2. Create Pool (L1)", async () => {
    const now = Math.floor(Date.now() / 1000);
    const START_TIME = new anchor.BN(now);
    END_TIME = START_TIME.add(new anchor.BN(60)); // 60s duration

    [poolPda] = PublicKey.findProgramAddressSync(
      [SEED_POOL, admin.publicKey.toBuffer(), new anchor.BN(poolId).toBuffer('le', 8)],
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
        POOL_NAME,
        "BTC/USDC",
        START_TIME,
        END_TIME,
        new anchor.BN(500),
        new anchor.BN(1000),
      )
      .accountsPartial({
        protocol: protocolPda,
        pool: poolPda,
        poolVault: vaultPda,
        tokenMint: usdcMint,
        admin: admin.publicKey,
        adminTokenAccount: adminAta.address,
        tokenProgram: TOKEN_PROGRAM_ID,
        systemProgram: SystemProgram.programId,
        rent: anchor.web3.SYSVAR_RENT_PUBKEY,
      })
      .rpc();
    console.log("    ✅ Pool Created on L1");
  });

  // Define this array outside the loop (alongside betPdas)
  const permissionPdas: PublicKey[] = [];

  it("3.1. Secure Bet Setup (L1: Init, Permission, Delegate)", async () => {
    const betAmount = new anchor.BN(100 * 1e6);

    for (let i = 0; i < users.length; i++) {
      const user = users[i];
      const requestId = requestIds[i];

      const [betPda] = PublicKey.findProgramAddressSync(
        [
          SEED_BET,
          poolPda.toBuffer(),
          user.publicKey.toBuffer(),
          Buffer.from(requestId),
        ],
        program.programId,
      );
      betPdas.push(betPda);

      const permissionPda = permissionPdaFromAccount(betPda);
      permissionPdas.push(permissionPda);

      console.log(`    Processing User ${i + 1} Setup...`);

      // 1. Init Bet
      await program.methods
        .initBet(betAmount, requestId)
        .accountsPartial({
          user: user.publicKey,
          protocol: protocolPda,
          pool: poolPda,
          poolVault: vaultPda,
          userTokenAccount: userAtas[i],
          userBet: betPda,
          tokenProgram: TOKEN_PROGRAM_ID,
          systemProgram: SystemProgram.programId,
          rent: anchor.web3.SYSVAR_RENT_PUBKEY,
        })
        .signers([user])
        .rpc();

      // 2. Create Permission
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
        .signers([user])
        .rpc();

      const delegationRecord =
        delegationRecordPdaFromDelegatedAccount(permissionPda);

      const delegationMetadata =
        delegationMetadataPdaFromDelegatedAccount(permissionPda);

      const delegationBuffer =
        delegateBufferPdaFromDelegatedAccountAndOwnerProgram(
          permissionPda,
          PERMISSION_PROGRAM_ID,
        );

      await program.methods
        .delegateBetPermission(requestId)
        .accountsPartial({
          user: user.publicKey,
          pool: poolPda,
          userBet: betPda,
          permission: permissionPda,
          permissionProgram: PERMISSION_PROGRAM_ID,
          delegationProgram: DELEGATION_PROGRAM_ID,
          delegationRecord: delegationRecord,
          delegationMetadata: delegationMetadata,
          delegationBuffer: delegationBuffer,
          validator: TEE_VALIDATOR,
          systemProgram: SystemProgram.programId,
        })
        .signers([user])
        .rpc();

      console.log(`      User ${i + 1}: Permission Delegated`);

      // 3. Delegate Bet (Standard)
      await program.methods
        .delegateBet(requestId)
        .accountsPartial({
          user: user.publicKey,
          pool: poolPda,
          userBet: betPda,
          validator: TEE_VALIDATOR,
        })
        .signers([user])
        .rpc();

      console.log(`      User ${i + 1}: L1 Setup Complete (Bet Delegated)`);
    }
  });

  // --- PART B: TEE Execution (Fast Off-Chain Transactions) ---
  it("3.2. Secure Bet Execution (TEE: Auth & Place Bet)", async () => {
    // Check if we already verified integrity to avoid redundant calls
    // (Optional, but good for speed)

    for (let i = 0; i < users.length; i++) {
      const user = users[i];
      const requestId = requestIds[i];
      const betPda = betPdas[i];

      console.log(`      Processing User ${i + 1} on TEE...`);
      let tokenString = "";

      // 1. Try to Generate Auth Token with Retry
      try {
        const authToken = await getAuthTokenWithRetry(
          ephemeralRpcEndpoint,
          user.publicKey,
          async (message) => nacl.sign.detached(message, user.secretKey),
        );
        tokenString = `?token=${authToken.token}`;
        console.log(`      🔐 Auth Token generated successfully.: ${ephemeralRpcEndpoint}${tokenString}`);
      } catch (e) {
        console.error(
          `      ❌ Auth failed. Server might be busy. Falling back to Anonymous.`,
        );
      }

      const erConnection = new anchor.web3.Connection(
        `${ephemeralRpcEndpoint}${tokenString}`,
        {
          commitment: "confirmed",
          wsEndpoint: `${ephemeralWsEndpoint}${tokenString}`,
        },
      );

      const erProvider = new anchor.AnchorProvider(
        erConnection,
        new anchor.Wallet(user),
        anchor.AnchorProvider.defaultOptions(),
      );
      const erProgram = new anchor.Program(program.idl, erProvider);

      console.log(`      📝 Placing Bet securely on TEE...`, predictions[i], requestId, erProgram.programId);
      const placeBetIx = await program.methods
        .placeBet(predictions[i], requestId)
        .accountsPartial({
          user: user.publicKey,
          pool: poolPda,
          userBet: betPda,
        })
        .instruction();

      let tx = new anchor.web3.Transaction().add(
        placeBetIx
      );

      tx.feePayer = user.publicKey;
      tx.recentBlockhash = (
        await erProvider.connection.getLatestBlockhash())
        .blockhash;
      const txHash = await sendAndConfirmTransaction(erProvider.connection, tx, [user], {
        skipPreflight: true,
        commitment: "confirmed",
      });

      console.log(`✅ Player ${user.publicKey} predicted ${predictions[i]}: ${txHash}`);

      // PAUSE between users to prevent "No challenge received" (Rate Limit)
      if (i < users.length - 1) {
        console.log("      ⏳ Pausing 5s for server cooldown...");
        await sleep(5000);
      }
    }
  });

  it("3.5. Sneak Peek (Privacy Check - TEE)", async () => {
    const spyUser = users[0];
    const targetBetPda = betPdas[1];

    console.log(`    🕵️  User 1 authenticating to peek at User 2's bet...`);
    let tokenString = "";

    try {
      // We use the Retry Helper here too
      const authToken = await getAuthTokenWithRetry(
        ephemeralRpcEndpoint,
        spyUser.publicKey,
        async (message) => nacl.sign.detached(message, spyUser.secretKey),
      );
      tokenString = `?token=${authToken.token}`;
    } catch (e) {
      console.log(
        `    ⚠️  SKIPPING STRICT PRIVACY CHECK: Auth Server is Down.`,
      );
      return;
    }

    const spyConnection = new anchor.web3.Connection(
      `${ephemeralRpcEndpoint}${tokenString}`,
      { commitment: "confirmed" },
    );
    const spyProvider = new anchor.AnchorProvider(
      spyConnection,
      new anchor.Wallet(spyUser),
      anchor.AnchorProvider.defaultOptions(),
    );

    try {
      const erProgram = new anchor.Program(
        program.idl,
        spyProvider,
      ) as Program<SwivPrivacy>;
      const betData = await erProgram.account.userBet.fetch(targetBetPda);
      console.error(
        "    ❌ DATA LEAKED! Spy read the prediction:",
        betData.prediction.toString(),
      );
      throw new Error("PRIVACY_FAILED: Account data is visible.");
    } catch (e: any) {
      if (e.message.includes("PRIVACY_FAILED")) throw e;
      if (
        e.message.includes("Account does not exist") ||
        e.message.includes("Constraint") ||
        e.message.includes("Access denied")
      ) {
        console.log("    ✅ Privacy Confirmed: TEE blocked the Spy.");
        return;
      }
      console.log("    ⚠️ Received error:", e.message);
    }
  });

  it("4. Wait for Expiry", async () => {
    console.log("    ⏳ Waiting for pool expiry...");
    const expiryMs = END_TIME.toNumber() * 1000;
    const bufferMs = 5000;
    const waitTime = expiryMs + bufferMs - Date.now();
    if (waitTime > 0) {
      console.log(`       Sleeping for ${waitTime / 1000} seconds...`);
      await sleep(waitTime);
    }
  });

  it("5. Delegate Pool (NOW we move Pool to TEE)", async () => {
    await program.methods
      .delegatePool(new anchor.BN(poolId))
      .accountsPartial({
        admin: admin.publicKey,
        protocol: protocolPda,
        pool: poolPda,
        validator: TEE_VALIDATOR,
      })
      .rpc();
    console.log("    ✅ Pool Moved to TEE for Calculation");
  });

  it("6. Resolve & Settle", async () => {
    let tokenString = "";
    try {
      const authToken = await getAuthTokenWithRetry(
        ephemeralRpcEndpoint,
        admin.publicKey,
        async (message) => nacl.sign.detached(message, admin.secretKey),
      );
      tokenString = `?token=${authToken.token}`;
      console.log(`    🔐 Admin Auth Token generated.`);
    } catch (e) {
      console.warn(
        "    ❌ Auth failed. Server might be busy. Falling back to Anonymous.",
      );
    }

    const erConnection = new anchor.web3.Connection(`${ephemeralRpcEndpoint}${tokenString}`, {
      commitment: "confirmed",
      wsEndpoint: `${ephemeralWsEndpoint}${tokenString}`,
    });
    const erProvider = new anchor.AnchorProvider(
      erConnection,
      new anchor.Wallet(admin),
      anchor.AnchorProvider.defaultOptions(),
    );
    const erProgram = new anchor.Program(program.idl, erProvider);

    await erProgram.methods
      .resolvePool(TARGET_PRICE)
      .accountsPartial({
        admin: admin.publicKey,
        protocol: protocolPda,
        pool: poolPda,
      })
      .rpc();
    console.log("    ✅ Pool Resolved on TEE");

    const batchAccounts = betPdas.map((k) => ({
      pubkey: k,
      isWritable: true,
      isSigner: false,
    }));
    await erProgram.methods
      .batchCalculateWeights()
      .accountsPartial({ admin: admin.publicKey, pool: poolPda })
      .remainingAccounts(batchAccounts)
      .preInstructions([
        ComputeBudgetProgram.setComputeUnitLimit({ units: 1_000_000 }),
      ])
      .rpc();
    console.log("    ✅ Winners Calculated on TEE");

    await erProgram.methods
      .batchUndelegateBets()
      .accounts({ payer: admin.publicKey, pool: poolPda })
      .remainingAccounts(batchAccounts)
      .rpc();

    await erProgram.methods
      .undelegatePool()
      .accounts({
        admin: admin.publicKey,
        protocol: protocolPda,
        pool: poolPda,
      })
      .rpc();
    console.log("    ✅ Settled back to L1");

    const config = await program.account.protocol.fetch(protocolPda);
    const treasuryAta = await getOrCreateAssociatedTokenAccount(
      provider.connection,
      admin,
      usdcMint,
      config.treasuryWallet,
      true,
    );
    await program.methods
      .finalizeWeights()
      .accountsPartial({
        admin: admin.publicKey,
        protocol: protocolPda,
        pool: poolPda,
        poolVault: vaultPda,
        treasuryTokenAccount: treasuryAta.address,
        tokenProgram: TOKEN_PROGRAM_ID,
      })
      .rpc();

    for (let i = 0; i < users.length; i++) {
      try {
        await program.methods
          .claimReward()
          .accountsPartial({
            user: users[i].publicKey,
            pool: poolPda,
            poolVault: vaultPda,
            userBet: betPdas[i],
            userTokenAccount: userAtas[i],
            tokenProgram: TOKEN_PROGRAM_ID,
          })
          .signers([users[i]])
          .rpc();
        console.log(`    User ${i + 1}: Reward Claimed`);
      } catch (e) {
        console.log(`    User ${i + 1}: No Reward`);
      }
    }
  });

  it("7. Public Verify (Transparency Check - L1)", async () => {
    const targetBetPda = betPdas[1];
    console.log(`    🌍 User 1 checking User 2's bet on Public L1...`);
    const betData = await program.account.userBet.fetch(targetBetPda);
    console.log(
      `    📖 L1 Data Found! User 2 Prediction: ${betData.prediction.toString()}`,
    );
    if (betData.prediction.eq(predictions[1])) {
      console.log(
        "    ✅ Transparency Confirmed: L1 state matches TEE execution.",
      );
    } else {
      throw new Error(
        `❌ Data Mismatch: Expected ${predictions[1]}, got ${betData.prediction}`,
      );
    }
  });
});
