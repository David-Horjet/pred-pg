import { PublicKey } from "@solana/web3.js";
import * as anchor from "@anchor-lang/core";

export const SEED_BET = Buffer.from("bet");
export const SEED_POOL = Buffer.from("pool");
export const SEED_POOL_VAULT = Buffer.from("pool_vault");
export const SEED_PROTOCOL = Buffer.from("protocol_v1");

// Dynamically check if we are on localnet
let isLocalnet = true;
try {
  const rpcEndpoint = anchor.AnchorProvider.env().connection.rpcEndpoint;
  isLocalnet = rpcEndpoint.includes("localhost") || rpcEndpoint.includes("127.0.0.1");
} catch (e) {
  // Default to localnet if provider is not configured
}

export const TEE_VALIDATOR = isLocalnet
  ? new PublicKey("mAGicPQYBMvcYveUZA5F5UNNwyHvfYh5xkLS2Fr1mev")
  : new PublicKey("MTEWGuqxUpYZGFJQcp8tLN7x5v9BSeoFHYWQQ3n3xzo");

export const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

export { 
  permissionPdaFromAccount, 
  verifyTeeRpcIntegrity, 
  getAuthToken, 
  PERMISSION_PROGRAM_ID, 
  DELEGATION_PROGRAM_ID, 
  MAGIC_PROGRAM_ID,
  MAGIC_CONTEXT_ID,
  EPHEMERAL_VAULT_ID,
  waitUntilPermissionActive,
  delegationRecordPdaFromDelegatedAccount, 
  delegationMetadataPdaFromDelegatedAccount, 
  delegateBufferPdaFromDelegatedAccountAndOwnerProgram 
} from "@magicblock-labs/ephemeral-rollups-sdk";