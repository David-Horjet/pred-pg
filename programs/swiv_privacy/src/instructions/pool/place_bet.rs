use anchor_lang::prelude::*;
use crate::state::{Pool, PoolStatus, Bet, BetStatus, Protocol};
use crate::constants::{SEED_BET, SEED_POOL, SEED_PROTOCOL};
use crate::errors::CustomError;

#[derive(Accounts)]
#[instruction(prediction: u64, request_id: String)]
pub struct PlaceBet<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [SEED_PROTOCOL],
        bump,
        constraint = !protocol.paused @ CustomError::Paused
    )]
    pub protocol: Box<Account<'info, Protocol>>,

    #[account(
        seeds = [SEED_POOL, pool.created_by.as_ref(), &(pool.pool_id.to_le_bytes())],
        bump = pool.bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        mut,
        seeds = [SEED_BET, pool.key().as_ref(), user.key().as_ref()],
        bump = bet.bump,
        constraint = bet.user_pubkey == user.key() @ CustomError::Unauthorized
    )]
    pub bet: Box<Account<'info, Bet>>,
}

pub fn place_bet(
    ctx: Context<PlaceBet>,
    prediction: u64, 
    _request_id: String,
) -> Result<()> {
    let bet = &mut ctx.accounts.bet;
    let pool = &ctx.accounts.pool;

    let clock = Clock::get()?;
    require!(
        pool.status == PoolStatus::Active || pool.status == PoolStatus::Upcoming,
        CustomError::MarketClosed
    );
    require!(clock.unix_timestamp < pool.cutoff_time, CustomError::MarketClosed);
    require!(bet.status == BetStatus::Active, CustomError::BetAlreadyInitialized);

    bet.prediction = prediction;
    bet.update_count = bet.update_count.checked_add(1).unwrap();

    Ok(())
}