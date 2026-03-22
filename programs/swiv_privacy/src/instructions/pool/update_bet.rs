use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::{Bet, Pool, BetStatus};
use crate::constants::{SEED_BET, SEED_POOL, SEED_POOL_VAULT};
use crate::errors::CustomError;
use crate::events::BetUpdated;

#[derive(Accounts)]
pub struct UpdateBet<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
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

    #[account(mut)]
    pub user_token_account: Option<Box<Account<'info, TokenAccount>>>,

    #[account(
        mut,
        seeds = [SEED_POOL_VAULT, pool.key().as_ref()],
        bump,
        token::authority = pool,
    )]
    pub pool_vault: Option<Box<Account<'info, TokenAccount>>>,

    pub token_program: Program<'info, Token>,
}

pub fn update_bet(
    ctx: Context<UpdateBet>,
    new_prediction: u64,
    additional_stake: u64,
) -> Result<()> {
    let bet = &mut ctx.accounts.bet;
    let pool = &mut ctx.accounts.pool;

    bet.update_count = bet.update_count.checked_add(1).unwrap();
    bet.prediction = new_prediction;

    // Handle optional stake increase
    if additional_stake > 0 {
        let user_token_account = ctx.accounts.user_token_account.as_ref()
            .ok_or(CustomError::Unauthorized)?;
        let pool_vault = ctx.accounts.pool_vault.as_ref()
            .ok_or(CustomError::Unauthorized)?;

        // Transfer additional stake to pool vault
        token::transfer(
            CpiContext::new(
                ctx.accounts.token_program.to_account_info(),
                Transfer {
                    from: user_token_account.to_account_info(),
                    to: pool_vault.to_account_info(),
                    authority: ctx.accounts.user.to_account_info(),
                },
            ),
            additional_stake,
        )?;

        // Update stake and pool volume
        bet.stake = bet.stake.checked_add(additional_stake).unwrap();
        pool.total_volume = pool.total_volume.checked_add(additional_stake).unwrap();

        msg!("Bet Updated: Prediction={}, Additional Stake Added={}", new_prediction, additional_stake);
    } else {
        msg!("Bet Updated: Prediction={}", new_prediction);
    }

    emit!(BetUpdated {
        bet_address: bet.key(),
        user: ctx.accounts.user.key(),
        pool_identifier: pool.title.clone(),
    });

    Ok(())
}