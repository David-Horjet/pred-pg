use crate::constants::{SEED_GLOBAL_CONFIG, SEED_POOL, SEED_POOL_VAULT};
use crate::errors::CustomError;
use crate::state::{GlobalConfig, Pool};
use crate::events::WeightsFinalized;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct FinalizeWeights<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        seeds = [SEED_GLOBAL_CONFIG],
        bump,
    )]
    pub global_config: Account<'info, GlobalConfig>,

    #[account(
        mut,
        seeds = [SEED_POOL, pool.name.as_bytes()],
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
    let config = &ctx.accounts.global_config;

    require!(pool.is_resolved, CustomError::SettlementTooEarly);
    require!(!pool.weight_finalized, CustomError::AlreadySettled);

    let total_assets = ctx.accounts.pool_vault.amount;
    let mut distributable_amount = total_assets;
    let mut fee_amount: u64 = 0;

    if config.protocol_fee_bps > 0 {
        fee_amount = (total_assets as u128)
            .checked_mul(config.protocol_fee_bps as u128)
            .unwrap()
            .checked_div(10000)
            .unwrap() as u64;

        if fee_amount > 0 {
            let name_bytes = pool.name.as_bytes();
            let bump = pool.bump;
            let seeds = &[SEED_POOL, name_bytes, &[bump]];
            let signer = &[&seeds[..]];

            token::transfer(
                CpiContext::new_with_signer(
                    ctx.accounts.token_program.to_account_info(),
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

    pool.vault_balance = distributable_amount; 
    pool.weight_finalized = true;

    emit!(WeightsFinalized {
        pool_name: pool.name.clone(),
        total_weight: pool.total_weight,
        fee_deducted: fee_amount,
    });

    Ok(())
}