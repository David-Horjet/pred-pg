use anchor_lang::prelude::*;

#[derive(AnchorSerialize, AnchorDeserialize, Clone, PartialEq, Eq)]
pub enum PoolStatus {
    Upcoming,
    Active,
    Closed,
    Resolving,
    Resolved,
    Settled,
    Cancelled,
}

#[account]
pub struct Pool {
    pub created_by: Pubkey,
    pub title: String,
    pub pool_id: u64,
    pub stake_token_mint: Pubkey,

    pub start_time: i64,
    pub end_time: i64,
    pub cutoff_time: i64,

    /// Running total of all tokens deposited into the vault.
    pub total_staked: u64,
    /// Set by finalize_weights: total_staked minus protocol fee. Used for payout math.
    pub distributable_amount: u64,

    pub max_accuracy_buffer: u64,
    pub conviction_bonus_bps: u64,

    pub resolution_result: u64,
    pub resolution_ts: i64,

    pub total_weight: u128,
    pub total_participants: u64,
    /// Number of bets that have been scored by batch_calculate_weights for this pool's resolution.
    pub weights_calculated_count: u64,

    pub status: PoolStatus,

    pub bump: u8,
}