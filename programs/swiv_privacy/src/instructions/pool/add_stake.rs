use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};
use crate::state::Pool;
use crate::constants::{SEED_POOL, SEED_POOL_VAULT};

#[derive(Accounts)]
pub struct AddStake<'info> {
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
        seeds = [SEED_POOL_VAULT, pool.key().as_ref()],
        bump,
        token::authority = pool,
    )]
    pub pool_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub user_token_account: Box<Account<'info, TokenAccount>>,

    pub token_program: Program<'info, Token>,
}

pub fn add_stake(ctx: Context<AddStake>, amount: u64) -> Result<()> {
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

    ctx.accounts.pool.total_volume = ctx.accounts.pool.total_volume
        .checked_add(amount)
        .unwrap();

    Ok(())
}
