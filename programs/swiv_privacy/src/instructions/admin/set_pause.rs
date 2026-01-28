use anchor_lang::prelude::*;
use crate::state::Protocol;
use crate::constants::SEED_PROTOCOL;
use crate::errors::CustomError;
use crate::events::PauseChanged;

#[derive(Accounts)]
pub struct SetPause<'info> {
    #[account(
        mut,
        seeds = [SEED_PROTOCOL],
        bump,
        constraint = protocol.admin == admin.key() @ CustomError::Unauthorized
    )]
    pub protocol: Account<'info, Protocol>,

    pub admin: Signer<'info>,
}

pub fn set_pause(ctx: Context<SetPause>, paused: bool) -> Result<()> {
    ctx.accounts.protocol.paused = paused;

    emit!(PauseChanged {
        is_paused: paused,
    });

    Ok(())
}