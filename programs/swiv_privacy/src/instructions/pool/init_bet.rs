use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::{Protocol, Pool, Bet, BetStatus};
use crate::constants::{SEED_BET, SEED_POOL, SEED_POOL_VAULT, SEED_PROTOCOL}; 
use crate::errors::CustomError;

#[derive(Accounts)]
#[instruction(amount: u64, request_id: String)]
pub struct InitBet<'info> {
    /// The user who owns the bet and authorizes the USDC transfer.
    /// Does NOT pay for account creation — sponsor covers rent.
    #[account(mut)]
    pub user: Signer<'info>,

    /// The protocol authority (sponsor) that pays for the bet account rent.
    /// Constrained to protocol.admin so only the authorized sponsor can be used.
    #[account(
        mut,
        constraint = sponsor.key() == protocol.admin @ CustomError::Unauthorized
    )]
    pub sponsor: Signer<'info>,

    #[account(
        seeds = [SEED_PROTOCOL],
        bump,
        constraint = !protocol.paused @ CustomError::Paused
    )]
    pub protocol: Box<Account<'info, Protocol>>,

    #[account(
        mut,
        seeds = [SEED_POOL, pool.created_by.as_ref(), &(pool.pool_id.to_le_bytes())],
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
        payer = sponsor,
        space = Bet::SPACE,
        seeds = [SEED_BET, pool.key().as_ref(), user.key().as_ref()],
        bump
    )]
    pub bet: Box<Account<'info, Bet>>,

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

    pool.total_volume = pool.total_volume.checked_add(amount).unwrap();
    pool.total_participants = pool.total_participants.checked_add(1).unwrap();

    let bet = &mut ctx.accounts.bet;
    bet.user_pubkey = ctx.accounts.user.key();
    bet.pool_pubkey = pool_key;
    bet.stake = amount; 
    bet.end_timestamp = pool.end_time;
    bet.creation_ts = clock.unix_timestamp; 
    bet.update_count = 0;                   
    bet.calculated_weight = 0;
    bet.is_weight_added = false;
    
    bet.status = BetStatus::Active;
    bet.prediction = 0; 
    bet.bump = ctx.bumps.bet;

    msg!("Bet Initialized on L1. Funds Secured.");

    Ok(())
}