use anchor_lang::prelude::*;

use crate::{GameError, GlobalState};

/// Enum representing the possible updates to the global state.
#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub enum GlobalStateUpdate {
    Admin(Pubkey),
    MessageSigner(Pubkey),
    MaxDeposit(u8),
    MaxPayout(u8),
}

/// Arguments for updating the global state.
#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct UpdateGlobalStateArgs {
    pub update: GlobalStateUpdate,
}

#[derive(Accounts)]
#[instruction(args: UpdateGlobalStateArgs)]
pub struct UpdateGlobalStateCtx<'info> {
    #[account(
        mut,
        seeds = [b"global-state"],
        bump = global_state.get_bump(),
    )]
    pub global_state: Account<'info, GlobalState>,

    /// Only the admin can update the global state.
    pub admin: Signer<'info>,
}

#[inline(always)] // This function is only called once, in the handler.
/// Perform the preliminary checks, other checks may be perfomed later in the handler.
fn checks(
    ctx: &Context<UpdateGlobalStateCtx>,
)->Result<()>{
    // Only the current admin can update the global state.
    require_keys_eq!(
        ctx.accounts.admin.key(),
        ctx.accounts.global_state.admin,
        GameError::InvalidAdmin
    );

    Ok(())
}

pub fn update_global_state_handler(
    ctx: Context<UpdateGlobalStateCtx>,
    args: UpdateGlobalStateArgs,
) -> Result<()> {

    checks(&ctx)?;

    let global_state = &mut ctx.accounts.global_state;

    match args.update {
        GlobalStateUpdate::Admin(new_admin) => {
            global_state.admin = new_admin;
        }
        GlobalStateUpdate::MessageSigner(new_signer) => {
            global_state.message_signer = new_signer;
        }
        GlobalStateUpdate::MaxDeposit(new_max_deposit) => {
            global_state.max_deposit = new_max_deposit;
        }
        GlobalStateUpdate::MaxPayout(new_max_payout) => {
            global_state.max_payout = new_max_payout;
        }
    }

    Ok(())
}