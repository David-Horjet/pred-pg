use anchor_lang::prelude::*;
use crate::state::{Bet, Protocol, Pool}; 
use crate::constants::{SEED_BET, SEED_POOL, SEED_PROTOCOL}; 
use crate::errors::CustomError;

use ephemeral_rollups_sdk::anchor::{delegate, commit};
use ephemeral_rollups_sdk::cpi::DelegateConfig;
use ephemeral_rollups_sdk::ephem::commit_and_undelegate_accounts;

#[delegate]
#[derive(Accounts)]
#[instruction(pool_id: u64)]
pub struct DelegatePool<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [SEED_PROTOCOL],
        bump,
        constraint = protocol.admin == admin.key() @ CustomError::Unauthorized
    )]
    pub protocol: Account<'info, Protocol>,

    /// CHECK: The main pool account.
   #[account(
        mut, 
        del, 
        seeds = [SEED_POOL, admin.key().as_ref(), &pool_id.to_le_bytes()],
        bump
    )]
    pub pool: UncheckedAccount<'info>,

    /// CHECK: The MagicBlock Ephemeral Rollup Validator (TEE)
    pub validator: UncheckedAccount<'info>,
}

pub fn delegate_pool<'info>(ctx: Context<'info, DelegatePool<'info>>, pool_id: u64) -> Result<()> {
    let admin_key = ctx.accounts.admin.key();
    let admin_bytes = admin_key.as_ref();
    let pool_id_bytes = pool_id.to_le_bytes();
    let seeds = &[
        SEED_POOL,
        admin_bytes,
        &pool_id_bytes,
    ];

    let config = DelegateConfig {
        validator: Some(ctx.accounts.validator.key()),
        ..DelegateConfig::default()
    };

    ctx.accounts.delegate_pool(
        &ctx.accounts.admin, 
        seeds, 
        config,             
    )?;

    msg!("Pool account delegated successfully.");
    Ok(())
}


#[delegate]
#[derive(Accounts)]
pub struct DelegateBet<'info> {
    /// The user who owns the bet — must sign to authorize delegation.
    #[account(mut)]
    pub user: Signer<'info>,

    /// Pays for any accounts the delegation SDK creates internally.
    /// Separated from user so users need zero SOL under the gas-sponsorship model.
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: Manually validated against the bet's pool_identifier.
    pub pool: UncheckedAccount<'info>,

    /// CHECK: The user's bet account.
    #[account(mut, del)]
    pub user_bet: UncheckedAccount<'info>,

    /// CHECK: The MagicBlock Ephemeral Rollup Validator (TEE)
    pub validator: UncheckedAccount<'info>,
}

pub fn delegate_bet<'info>(ctx: Context<'info, DelegateBet<'info>>) -> Result<()> {
    let (pool_pubkey, owner) = {
        let user_bet_data = ctx.accounts.user_bet.try_borrow_data()?;
        let mut data_slice: &[u8] = &user_bet_data;
        let bet = Bet::try_deserialize(&mut data_slice)?;
        (bet.pool_pubkey, bet.user_pubkey)
    }; 

    require!(owner == ctx.accounts.user.key(), CustomError::Unauthorized);
    require!(pool_pubkey == ctx.accounts.pool.key(), CustomError::PoolMismatch);

    let pool_key = ctx.accounts.pool.key();
    let user_key = ctx.accounts.user.key();

    let seeds_for_sdk = &[
        SEED_BET,
        pool_key.as_ref(), 
        user_key.as_ref(),
    ];
    
    let config = DelegateConfig {
        validator: Some(ctx.accounts.validator.key()),
        ..DelegateConfig::default()
    };

    ctx.accounts.delegate_user_bet(
        &ctx.accounts.payer,
        seeds_for_sdk,
        config,
    )?;

    msg!("Bet delegated successfully.");
    Ok(())
}

#[commit]
#[derive(Accounts)]
pub struct UndelegatePool<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [SEED_PROTOCOL],
        bump,
        constraint = protocol.admin == admin.key() @ CustomError::Unauthorized
    )]
    pub protocol: Account<'info, Protocol>,
    
    /// CHECK: The Pool account
    #[account(mut)]
    pub pool: UncheckedAccount<'info>,
}

pub fn undelegate_pool<'info>(ctx: Context<'info, UndelegatePool<'info>>) -> Result<()> {
    commit_and_undelegate_accounts(
        &ctx.accounts.admin,
        vec![&ctx.accounts.pool],
        &ctx.accounts.magic_context,
        &ctx.accounts.magic_program,
        None,
    )?;

    Ok(())
}

#[commit]
#[derive(Accounts)]
pub struct BatchUndelegateBets<'info> {
    #[account(mut)]
    pub payer: Signer<'info>, 

    #[account(
        mut,
        seeds = [SEED_POOL, pool.created_by.as_ref(), &(pool.pool_id.to_le_bytes())],
        bump = pool.bump
    )]
    pub pool: Account<'info, Pool>,
}

pub fn batch_undelegate_bets<'info>(ctx: Context<'info, BatchUndelegateBets<'info>>) -> Result<()> {
    let pool = &ctx.accounts.pool;
    let clock = Clock::get()?;

    require!(
        clock.unix_timestamp >= pool.end_time,
        CustomError::UndelegationTooEarly
    );
    
    let accounts_to_undelegate: Vec<&AccountInfo<'info>> = ctx.remaining_accounts.iter().collect();
    
    if accounts_to_undelegate.is_empty() {
        return Ok(());
    }

    commit_and_undelegate_accounts(
        &ctx.accounts.payer,
        accounts_to_undelegate,
        &ctx.accounts.magic_context,
        &ctx.accounts.magic_program,
        None,
    )?;

    msg!("Batch Undelegate executed for {} bets.", ctx.remaining_accounts.len());
    Ok(())
}

#[commit]
#[derive(Accounts)]
pub struct UndelegateBet<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: The user's bet account.
    #[account(mut)]
    pub user_bet: UncheckedAccount<'info>,
}

pub fn undelegate_bet<'info>(ctx: Context<'info, UndelegateBet<'info>>) -> Result<()> {
    commit_and_undelegate_accounts(
        &ctx.accounts.payer,
        vec![&ctx.accounts.user_bet],
        &ctx.accounts.magic_context,
        &ctx.accounts.magic_program,
        None,
    )?;
    Ok(())
}