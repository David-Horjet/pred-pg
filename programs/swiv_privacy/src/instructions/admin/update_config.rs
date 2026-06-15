use anchor_lang::prelude::*;
use crate::state::Protocol;
use crate::constants::{SEED_PROTOCOL, MAX_FEE_BPS};
use crate::errors::CustomError;
use crate::events::ConfigUpdated;

#[derive(Accounts)]
#[instruction(
    new_treasury: Option<Pubkey>,
    new_protocol_fee_bps: Option<u64>,
    new_batch_settle_wait_duration: Option<i64>,
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
    new_batch_settle_wait_duration: Option<i64>,
) -> Result<()> {
    let protocol = &mut ctx.accounts.protocol;

    if let Some(treasury) = new_treasury {
        protocol.treasury_wallet = treasury;
    }

    if let Some(fee) = new_protocol_fee_bps {
        require!(fee <= MAX_FEE_BPS, CustomError::InvalidFee);
        protocol.protocol_fee_bps = fee;
    }

    if let Some(duration) = new_batch_settle_wait_duration {
        protocol.batch_settle_wait_duration = duration;
    }

    emit!(ConfigUpdated {
        treasury: new_treasury,
        protocol_fee_bps: new_protocol_fee_bps,
        batch_settle_wait_duration: new_batch_settle_wait_duration,
    });

    msg!("Protocol Config Updated");

    Ok(())
}