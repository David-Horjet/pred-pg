use anchor_lang::prelude::*;
use crate::state::Protocol;
use crate::constants::SEED_PROTOCOL;
use crate::errors::CustomError;
use crate::events::AdminTransferred;

#[derive(Accounts)]
pub struct TransferAdmin<'info> {
    #[account(mut)]
    pub current_admin: Signer<'info>,

    #[account(
        mut,
        seeds = [SEED_PROTOCOL],
        bump,
        constraint = protocol.admin == current_admin.key() @ CustomError::Unauthorized
    )]
    pub protocol: Account<'info, Protocol>,
}

pub fn transfer_admin(ctx: Context<TransferAdmin>, new_admin: Pubkey) -> Result<()> {
    let protocol = &mut ctx.accounts.protocol;
    let old_admin = protocol.admin;
    
    protocol.admin = new_admin;
    
    emit!(AdminTransferred {
        old_admin,
        new_admin,
    });

    msg!("Admin transferred from {} to {}", old_admin, new_admin);

    Ok(())
}