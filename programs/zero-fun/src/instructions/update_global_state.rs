use anchor_lang::prelude::*;

use crate::{GameError, GlobalState, GlobalStateUpdate, UpdateGlobalStateEvent};


#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct UpdateGlobalStateArgs {
    pub update: GlobalStateUpdate,
}

#[derive(Accounts)]
#[instruction(args: UpdateGlobalStateArgs)]
pub struct UpdateGlobalStateAccounts<'info> {
    pub global_state: Account<'info, GlobalState>,

    /// Only the admin can update the global state.
    pub admin: Signer<'info>,
}

#[inline(always)]
fn checks(
    ctx: &Context<UpdateGlobalStateAccounts>,
)->Result<()>{
    // Only the current admin can update the global state.
    require!(
        ctx.accounts.global_state.is_admin(ctx.accounts.admin.key),
        GameError::InvalidAdmin
    );

    Ok(())
}

pub fn update_global_state_handler(
    ctx: Context<UpdateGlobalStateAccounts>,
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
        },
        GlobalStateUpdate::GameState(new_game_state) => {
            global_state.game_state = new_game_state;
        }
    }

    emit!(
        UpdateGlobalStateEvent{
            admin_at_time_of_update:ctx.accounts.admin.key(),
            update: args.update
        }
    );

    Ok(())
}