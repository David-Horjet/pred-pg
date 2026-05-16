use anchor_lang::prelude::*;
use crate::state::{Pool, PoolStatus, Protocol};
use crate::constants::{SEED_PROTOCOL, SEED_POOL};
use crate::errors::CustomError;

#[derive(Accounts)]
pub struct CancelPool<'info> {
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

pub fn cancel_pool(ctx: Context<CancelPool>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;

    require!(
        pool.status == PoolStatus::Upcoming
            || pool.status == PoolStatus::Active
            || pool.status == PoolStatus::Closed,
        CustomError::PoolNotCancellable
    );

    pool.status = PoolStatus::Cancelled;

    msg!("Pool {} cancelled by admin.", pool.pool_id);

    Ok(())
}
