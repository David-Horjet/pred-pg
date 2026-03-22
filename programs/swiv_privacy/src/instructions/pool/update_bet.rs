use anchor_lang::prelude::*;
use crate::state::{Bet, Pool, BetStatus};
use crate::constants::{SEED_BET, SEED_POOL};
use crate::errors::CustomError;
use crate::events::BetUpdated;

/// TEE-only instruction: updates prediction and optionally records a stake increase.
/// Token transfers for stake increases MUST be handled separately via `add_stake` on L1
/// before calling this instruction, since the pool vault lives on L1 (not delegated to TEE).
#[derive(Accounts)]
pub struct UpdateBet<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    /// Pool is read-only here — only the bet account (delegated to TEE) is mutated.
    #[account(
        seeds = [SEED_POOL, pool.created_by.as_ref(), &(pool.pool_id.to_le_bytes())],
        bump = pool.bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        mut,
        seeds = [SEED_BET, pool.key().as_ref(), user.key().as_ref()],
        bump = bet.bump,
        constraint = bet.user_pubkey == user.key() @ CustomError::Unauthorized,
        constraint = bet.pool_pubkey == pool.key() @ CustomError::PoolMismatch,
        constraint = bet.status == BetStatus::Active @ CustomError::AlreadyClaimed
    )]
    pub bet: Box<Account<'info, Bet>>,
}

pub fn update_bet(
    ctx: Context<UpdateBet>,
    new_prediction: u64,
    additional_stake: u64,
) -> Result<()> {
    let bet = &mut ctx.accounts.bet;

    bet.update_count = bet.update_count.checked_add(1).unwrap();
    bet.prediction = new_prediction;

    if additional_stake > 0 {
        bet.stake = bet.stake.checked_add(additional_stake).unwrap();
        msg!("Bet Updated: Prediction={}, Stake+={}", new_prediction, additional_stake);
    } else {
        msg!("Bet Updated: Prediction={}", new_prediction);
    }

    emit!(BetUpdated {
        bet_address: bet.key(),
        user: ctx.accounts.user.key(),
        pool_identifier: ctx.accounts.pool.title.clone(),
    });

    Ok(())
}