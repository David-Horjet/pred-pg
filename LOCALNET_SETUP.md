# Localnet Testing Setup Guide

This guide details how to configure and run the privacy prediction protocol on **localnet**. 

Since the protocol depends on **Magicblock's Ephemeral Rollups** to handle private execution in TEEs, running tests locally requires running both a local Solana base validator and a local Ephemeral validator sequencer.

---

## 1. Prerequisites & Installation

To run the local Ephemeral Rollup validator stack, you must install the global Magicblock CLI utility via npm:

```bash
npm install -g @magicblock-labs/ephemeral-validator@latest
```

Ensure you have your program binaries (`.so` files) dumped from Devnet to the project root:

```bash
# 1. Dump the Delegation Program (DELeGGvX...)
solana program dump -u d DELeGGvXpWV2fqJUhqcF5ZSYMS4JTLjteaAMARRSaeSh delegation_program.so

# 2. Dump the Permission Program (ACLseoPo...)
solana program dump -u d ACLseoPoyC3cBqoUtkbjZ4aDrkurZW86v19pXz2XQnp1 permission_program.so
```

---

## 2. Running the Local Test Environment

You will need to open **three separate terminals** to run the localnet test suite:

### 🖥️ Terminal 1: Start the Solana Base Layer Validator
Start the custom Magicblock Solana test validator (which pre-loads necessary configurations and reads genesis settings from `Anchor.toml`):

```bash
mb-test-validator --reset
```
*Leave this terminal open and running.*

### 🖥️ Terminal 2: Start the Ephemeral Validator Sequencer
Once Terminal 1 is fully active, start the ephemeral validator in a second terminal to handle private transactions on the rollup:

```bash
ephemeral-validator --remotes "http://localhost:8899" -l "127.0.0.1:7799" --lifecycle ephemeral --no-tui --reset
```
*Leave this terminal open and running.*

### 🖥️ Terminal 3: Build, Deploy, and Run the Tests
Now that the validators are running, prepare the base layer and execute the tests:

```bash
# 1. Request SOL airdrop to deploy programs & cover transaction fees
solana airdrop 10

# 2. Deploy your program onto the running localnet
anchor deploy

# 3. Run the test suite
anchor run test
```

> [!CAUTION]
> Do NOT use `anchor test` directly! Since we are running the validators manually in Terminal 1 and 2, running `anchor test` will try to boot up its own validator, resulting in port conflicts. Always use **`anchor run test`**.

---

## 3. Tear Down & Cleanup (Crucial)

After you finish running a test suite, you **must clean up the database state** before running again. If you skip this, subsequent test runs will fail because the Ephemeral validator's database will get out of sync with the restarted Solana base layer.

Follow these cleanup steps:

1. Press `Ctrl + C` in **Terminal 2** to stop the `ephemeral-validator` process.
2. In your project root folder, delete the local Magicblock state folder:
   ```bash
   rm -rf magicblock-test-storage/
   ```
3. Restart **Terminal 2** (`ephemeral-validator`) before running the tests again.
