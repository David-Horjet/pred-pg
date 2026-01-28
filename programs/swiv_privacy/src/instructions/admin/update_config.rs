use anchor_lang::prelude::*;
use crate::state::Protocol;
use crate::constants::SEED_PROTOCOL;
use crate::errors::CustomError;
use crate::events::ConfigUpdated;

#[derive(Accounts)]
#[instruction(
    new_treasury: Option<Pubkey>, 
    new_protocol_fee_bps: Option<u64> 
)]
pub struct UpdateConfig<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        seeds = [SEED_PROTOCOL],
        bump,
        constraint = protocol.admin == admin.key() @ CustomError::Unauthorized,
    )]
    pub protocol: Account<'info, Protocol>,

    pub system_program: Program<'info, System>,
}

pub fn update_config(
    ctx: Context<UpdateConfig>,
    new_treasury: Option<Pubkey>,
    new_protocol_fee_bps: Option<u64>,
) -> Result<()> {
    let protocol = &mut ctx.accounts.protocol;

    if let Some(treasury) = new_treasury {
        protocol.treasury_wallet = treasury;
    }

    if let Some(fee) = new_protocol_fee_bps {
        protocol.protocol_fee_bps = fee;
    }

    emit!(ConfigUpdated {
        treasury: new_treasury,
        protocol_fee_bps: new_protocol_fee_bps,
    });

    msg!("Protocol Config Updated");

    Ok(())
}