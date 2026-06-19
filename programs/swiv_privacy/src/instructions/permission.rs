use crate::constants::SEED_BET;
use crate::errors::CustomError;
use anchor_lang::prelude::*;

use ephemeral_rollups_sdk::access_control::instructions::{CreateEphemeralPermissionCpi, CloseEphemeralPermissionCpi};
use ephemeral_rollups_sdk::access_control::structs::{Member, EphemeralMembersArgs, AUTHORITY_FLAG};

#[derive(Accounts)]
pub struct CreateBetPermission<'info> {
    pub payer: Signer<'info>,

    /// CHECK: The user who is given authority
    pub user: UncheckedAccount<'info>,

    /// CHECK: We manually verify seeds below to invoke with canonical bump
    #[account(mut)]
    pub user_bet: UncheckedAccount<'info>,

    /// CHECK: Passed to permission program. Must be UncheckedAccount.
    pub pool: UncheckedAccount<'info>,

    /// CHECK: Validated by Permission Program
    #[account(mut)]
    pub permission: UncheckedAccount<'info>,

    /// CHECK: The MagicBlock Permission Program ID
    pub permission_program: UncheckedAccount<'info>,

    /// CHECK: The MagicBlock Ephemeral Vault ID
    #[account(mut)]
    pub vault: UncheckedAccount<'info>,

    /// CHECK: The MagicBlock Program ID
    pub magic_program: UncheckedAccount<'info>,
}

pub fn create_bet_permission(ctx: Context<CreateBetPermission>, _request_id: String) -> Result<()> {
    let pool_key = ctx.accounts.pool.key();
    let user_key = ctx.accounts.user.key();

    let seeds_no_bump: Vec<Vec<u8>> = vec![
        SEED_BET.to_vec(),
        pool_key.to_bytes().to_vec(),
        user_key.to_bytes().to_vec(),
    ];

    let (derived_pda, bump) = Pubkey::find_program_address(
        &seeds_no_bump
            .iter()
            .map(|s| s.as_slice())
            .collect::<Vec<_>>(),
        &crate::ID,
    );

    require!(
        derived_pda == ctx.accounts.user_bet.key(),
        CustomError::SeedMismatch
    );

    let mut seeds = seeds_no_bump.clone();
    seeds.push(vec![bump]);
    let seed_refs: Vec<&[u8]> = seeds.iter().map(|s| s.as_slice()).collect();
    let signer_seeds = &[seed_refs.as_slice()];

    let member = Member {
        pubkey: ctx.accounts.user.key(),
        flags: AUTHORITY_FLAG,
    };
    let args = EphemeralMembersArgs {
        is_private: true,
        members: vec![member],
    };

    let cpi = CreateEphemeralPermissionCpi {
        permissioned_account: ctx.accounts.user_bet.to_account_info(),
        permission: ctx.accounts.permission.to_account_info(),
        payer: ctx.accounts.user_bet.to_account_info(),
        vault: ctx.accounts.vault.to_account_info(),
        magic_program: ctx.accounts.magic_program.to_account_info(),
        permission_program: ctx.accounts.permission_program.to_account_info(),
        args,
    };

    cpi.invoke_signed(signer_seeds)?;

    Ok(())
}

use crate::state::Bet;

#[derive(Accounts)]
pub struct CloseBetPermission<'info> {
    #[account(mut)]
    pub payer: Signer<'info>,

    /// CHECK: The user's bet account (the permissioned account) which acts as the authority
    #[account(mut)]
    pub user_bet: UncheckedAccount<'info>,

    /// CHECK: The permission account to close
    #[account(mut)]
    pub permission: UncheckedAccount<'info>,

    /// CHECK: The MagicBlock Ephemeral Vault ID
    #[account(mut)]
    pub vault: UncheckedAccount<'info>,

    /// CHECK: The MagicBlock Program ID
    pub magic_program: UncheckedAccount<'info>,

    /// CHECK: The MagicBlock Permission Program ID
    pub permission_program: UncheckedAccount<'info>,
}

pub fn close_bet_permission(ctx: Context<CloseBetPermission>) -> Result<()> {
    let (pool_pubkey, user_pubkey, bump) = {
        let user_bet_data = ctx.accounts.user_bet.try_borrow_data()?;
        let mut data_slice: &[u8] = &user_bet_data;
        let bet = Bet::try_deserialize(&mut data_slice)?;
        (bet.pool_pubkey, bet.user_pubkey, bet.bump)
    };

    let seeds_for_signing = &[
        SEED_BET,
        pool_pubkey.as_ref(),
        user_pubkey.as_ref(),
        &[bump],
    ];
    let signer_seeds = &[&seeds_for_signing[..]];

    let cpi = CloseEphemeralPermissionCpi {
        permissioned_account: ctx.accounts.user_bet.to_account_info(),
        permission: ctx.accounts.permission.to_account_info(),
        payer: ctx.accounts.user_bet.to_account_info(),
        authority: ctx.accounts.user_bet.to_account_info(),
        vault: ctx.accounts.vault.to_account_info(),
        magic_program: ctx.accounts.magic_program.to_account_info(),
        permission_program: ctx.accounts.permission_program.to_account_info(),
        authority_is_signer: false,
    };

    cpi.invoke_signed(signer_seeds)?;

    Ok(())
}
