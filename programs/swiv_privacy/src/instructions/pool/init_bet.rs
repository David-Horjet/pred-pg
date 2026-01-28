use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::{Protocol, Pool, UserBet, BetStatus};
use crate::constants::{SEED_BET, SEED_POOL, SEED_POOL_VAULT, SEED_PROTOCOL}; 
use crate::errors::CustomError;

#[derive(Accounts)]
#[instruction(amount: u64, request_id: String)]
pub struct InitBet<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        seeds = [SEED_PROTOCOL],
        bump,
        constraint = !protocol.paused @ CustomError::Paused
    )]
    pub protocol: Box<Account<'info, Protocol>>,

    #[account(
        mut,
        seeds = [SEED_POOL, pool.admin.as_ref(), &(pool.pool_id.to_le_bytes())],
        bump = pool.bump
    )]
    pub pool: Box<Account<'info, Pool>>,

    #[account(
        mut,
        seeds = [SEED_POOL_VAULT, pool.key().as_ref()],
        bump,
        token::authority = pool,
    )]
    pub pool_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub user_token_account: Box<Account<'info, TokenAccount>>,

    #[account(
        init,
        payer = user,
        space = UserBet::SPACE,
        seeds = [SEED_BET, pool.key().as_ref(), user.key().as_ref(), request_id.as_bytes()], 
        bump
    )]
    pub user_bet: Box<Account<'info, UserBet>>,

    pub token_program: Program<'info, Token>,
    pub system_program: Program<'info, System>,
    pub rent: Sysvar<'info, Rent>,
}

pub fn init_bet(
    ctx: Context<InitBet>,
    amount: u64,
    _request_id: String, 
) -> Result<()> {
    let pool_key = ctx.accounts.pool.key();
    let pool = &mut ctx.accounts.pool;
    let clock = Clock::get()?;

    require!(clock.unix_timestamp >= pool.start_time, CustomError::DurationTooShort);
    require!(clock.unix_timestamp < pool.end_time, CustomError::DurationTooShort); 

    token::transfer(
        CpiContext::new(
            ctx.accounts.token_program.to_account_info(),
            Transfer {
                from: ctx.accounts.user_token_account.to_account_info(),
                to: ctx.accounts.pool_vault.to_account_info(),
                authority: ctx.accounts.user.to_account_info(),
            },
        ),
        amount,
    )?;

    pool.vault_balance = pool.vault_balance.checked_add(amount).unwrap();
    pool.total_participants = pool.total_participants.checked_add(1).unwrap();

    let user_bet = &mut ctx.accounts.user_bet;
    user_bet.owner = ctx.accounts.user.key();
    user_bet.pool = pool_key;
    user_bet.deposit = amount; 
    user_bet.end_timestamp = pool.end_time;
    user_bet.creation_ts = clock.unix_timestamp; 
    user_bet.update_count = 0;                   
    user_bet.calculated_weight = 0;
    user_bet.is_weight_added = false;
    
    user_bet.status = BetStatus::Initialized;
    user_bet.prediction = 0; 
    user_bet.bump = ctx.bumps.user_bet;

    msg!("Bet Initialized on L1. Funds Secured.");

    Ok(())
}