use anchor_lang::prelude::*;

#[event]
pub struct ProtocolInitialized {
    pub admin: Pubkey,
    pub fee_wallet: Pubkey,
}

#[event]
pub struct ConfigUpdated {
    pub treasury: Option<Pubkey>,
    pub protocol_fee_bps: Option<u64>,
    pub batch_settle_wait_duration: Option<i64>,
}

#[event]
pub struct PoolCreated {
    pub pool_name: String,
    pub start_time: i64,
    pub end_time: i64,
}

#[event]
pub struct RewardClaimed {
    pub bet_address: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
}

#[event]
pub struct BetRefunded {
    pub bet_address: Pubkey,
    pub user: Pubkey,
    pub amount: u64,
    pub is_emergency: bool,
}