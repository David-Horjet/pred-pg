use crate::constants::{SEED_POOL, SEED_POOL_VAULT};
use crate::errors::CustomError;
use crate::state::{BetStatus, Pool, PoolStatus, Bet};
use crate::events::RewardClaimed;
use anchor_lang::prelude::*;
use anchor_spl::token::{self, Token, TokenAccount, Transfer};

#[derive(Accounts)]
pub struct ClaimReward<'info> {
    #[account(mut)]
    pub user: Signer<'info>,

    #[account(
        mut,
        constraint = sponsor.key() == pool.created_by @ CustomError::Unauthorized
    )]
    pub sponsor: Signer<'info>,

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
    pub pool_vault: Account<'info, TokenAccount>,

    #[account(
        mut,
        close = sponsor,
        constraint = bet.user_pubkey == user.key() @ CustomError::Unauthorized,
        constraint = bet.pool_pubkey == pool.key() @ CustomError::PoolMismatch,
        constraint = bet.status != BetStatus::Claimed @ CustomError::AlreadyClaimed,
    )]
    pub bet: Box<Account<'info, Bet>>,

    #[account(mut)]
    pub user_token_account: Account<'info, TokenAccount>,

    pub token_program: Program<'info, Token>,
}

pub fn claim_reward(ctx: Context<ClaimReward>) -> Result<()> {
    let pool = &mut ctx.accounts.pool;
    let bet = &mut ctx.accounts.bet;
    let mut payout_amount: u64 = 0;

    require!(pool.status == PoolStatus::Resolved, CustomError::SettlementTooEarly);
 
    if pool.total_participants == 1 {
        payout_amount = ctx.accounts.pool_vault.amount;
    } else if pool.total_weight > 0 {
        require!(
            bet.status == BetStatus::Resolved,
            CustomError::NotCalculatedYet
        );
 
        if bet.calculated_weight > 0 {
            let total_distributable_pot = pool.distributable_amount as u128;
 
            payout_amount = bet
                .calculated_weight
                .checked_mul(total_distributable_pot)
                .unwrap()
                .checked_div(pool.total_weight)
                .unwrap() as u64;
        }
    } else {
        payout_amount = bet.stake;
    }

    if payout_amount > 0 {
        require!(
            payout_amount <= pool.distributable_amount,
            CustomError::InsufficientLiquidity
        );
        require!(
            payout_amount <= ctx.accounts.pool_vault.amount,
            CustomError::InsufficientLiquidity
        );

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
                    to: ctx.accounts.user_token_account.to_account_info(),
                    authority: pool.to_account_info(),
                },
                signer,
            ),
            payout_amount,
        )?;
        
    }

    bet.status = BetStatus::Claimed;

    emit!(RewardClaimed {
        bet_address: bet.key(),
        user: ctx.accounts.user.key(),
        amount: payout_amount,
    });

    // Decrement total participants and clean up pool accounts if all participants have claimed
    pool.total_participants = pool.total_participants.saturating_sub(1);
    if pool.total_participants == 0 {
        // 1. Close the pool vault token account
        let created_by_bytes = pool.created_by.as_ref();
        let pool_id_bytes = pool.pool_id.to_le_bytes();
        let bump = pool.bump;
        let seeds = &[SEED_POOL, created_by_bytes, &pool_id_bytes, &[bump]];
        let signer = &[&seeds[..]];

        token::close_account(
            CpiContext::new_with_signer(
                ctx.accounts.token_program.key(),
                token::CloseAccount {
                    account: ctx.accounts.pool_vault.to_account_info(),
                    destination: ctx.accounts.sponsor.to_account_info(),
                    authority: pool.to_account_info(),
                },
                signer,
            ),
        )?;

        // 2. Close the pool account itself by draining its lamports
        let pool_info = pool.to_account_info();
        let sponsor_info = ctx.accounts.sponsor.to_account_info();
        
        let sponsor_lamports = sponsor_info.lamports();
        let pool_lamports = pool_info.lamports();
        
        **sponsor_info.lamports.borrow_mut() = sponsor_lamports.checked_add(pool_lamports).unwrap();
        **pool_info.lamports.borrow_mut() = 0;

        msg!("All claims completed. Pool and vault accounts closed, rent reclaimed.");
    }

    Ok(())
}