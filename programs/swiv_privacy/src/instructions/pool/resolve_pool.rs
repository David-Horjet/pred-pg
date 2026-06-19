use anchor_lang::prelude::*;
use crate::state::{Pool, PoolStatus, Protocol};
use crate::constants::{SEED_PROTOCOL, SEED_POOL};
use crate::errors::CustomError;

#[derive(Accounts)]
pub struct ResolvePool<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [SEED_PROTOCOL],
        bump,
        constraint = protocol.admin == admin.key() @ CustomError::Unauthorized
    )]
    pub protocol: Account<'info, Protocol>,

    #[account(
        mut,
        seeds = [SEED_POOL, pool.created_by.as_ref(), &(pool.pool_id.to_le_bytes())],
        bump = pool.bump
    )]
    pub pool: Account<'info, Pool>,
}

/// `final_outcome == 0` is a valid, intentional value: calculate_accuracy_score
/// treats result == 0 as "no winners", giving every bet a weight of 0 and
/// causing claim_reward to refund each participant's full stake. Admins should
/// use 0 to void a pool (e.g. bad/unavailable oracle data) and resolve normally otherwise.
pub fn resolve_pool(ctx: Context<ResolvePool>, final_outcome: u64) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    require!(
        pool.status != PoolStatus::Resolved
            && pool.status != PoolStatus::Settled
            && pool.status != PoolStatus::Cancelled,
        CustomError::AlreadyResolved
    );

    let clock = Clock::get()?;
    require!(clock.unix_timestamp >= pool.end_time, CustomError::DurationTooShort);

    pool.resolution_result = final_outcome;
    pool.resolution_ts = clock.unix_timestamp;
    pool.status = PoolStatus::Resolving;

    msg!("Pool Resolving. Outcome: {}", final_outcome);

    Ok(())
}