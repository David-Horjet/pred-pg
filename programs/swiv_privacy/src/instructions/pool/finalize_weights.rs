use crate::constants::{SEED_POOL, SEED_POOL_VAULT, SEED_PROTOCOL};
use crate::errors::CustomError;
use crate::state::{Pool, PoolStatus, Protocol};
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct FinalizeWeights<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [SEED_PROTOCOL],
        bump,
    )]
    pub protocol: Account<'info, Protocol>,

    #[account(
        mut,
        seeds = [SEED_POOL, pool.created_by.as_ref(), &(pool.pool_id.to_le_bytes())],
        bump = pool.bump,
    )]
    pub pool: Account<'info, Pool>,

    #[account(
        mut,
        seeds = [SEED_POOL_VAULT, pool.key().as_ref()],
        bump,
        token::authority = pool,
    )]
    pub pool_vault: Account<'info, TokenAccount>,

    #[account(mut)]
    pub treasury_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn finalize_weights(ctx: Context<FinalizeWeights>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let config = &ctx.accounts.protocol;

    require!(pool.status == PoolStatus::Resolving, CustomError::SettlementTooEarly);

    require!(
        pool.weights_calculated_count == 0
            || pool.weights_calculated_count == pool.total_participants,
        CustomError::WeightsIncomplete
    );

    let clock = Clock::get()?;
    require!(
        clock.unix_timestamp >= pool.resolution_ts + config.batch_settle_wait_duration,
        CustomError::SettlementTooEarly
    );

    if pool.total_participants <= 1 {
        pool.total_weight = 0;
    }

    let total_assets = ctx.accounts.pool_vault.amount;
    let mut distributable_amount = total_assets;

    if config.protocol_fee_bps > 0 && pool.total_participants > 1 && pool.total_weight > 0 {
        let fee_amount = (total_assets as u128)
            .checked_mul(config.protocol_fee_bps as u128)
            .unwrap()
            .checked_div(10000)
            .unwrap() as u64;

        if fee_amount > 0 {
            let created_by_bytes = pool.created_by.as_ref();
            let pool_id_bytes = pool.pool_id.to_le_bytes();
            let bump = pool.bump;
            let seeds = &[SEED_POOL, created_by_bytes, &pool_id_bytes, &[bump]];
            let signer = &[&seeds[..]];

            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.key(),
                    Transfer {
                        from: ctx.accounts.pool_vault.to_account_info(),
                        to: ctx.accounts.treasury_token_account.to_account_info(),
                        authority: pool.to_account_info(),
                    },
                    signer,
                ),
                fee_amount,
            )?;

            distributable_amount = total_assets.checked_sub(fee_amount).unwrap();
        }
    }

    pool.distributable_amount = distributable_amount;
    pool.status = PoolStatus::Resolved;

    Ok(())
}
