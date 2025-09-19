use anchor_lang::{
    prelude::*,
    solana_program::{
        sysvar::instructions::ID as INSTRUCTIONS_SYSVAR_ADDRESS
    }
};

use crate::{
    FINALIZE_LOSS_ACTION, GameError, GameSession, GameStatus, 
    GlobalState, HASH_LENGTH, MAX_MOVE_COUNT, is_signature_valid
};

/// Arguments for finalizing a game session as a loss.
/// - private_config_seed: The SHA-256 hash seed used to derive the private configuration
///   of the game.
/// - finalized_game_state: The final state of the game represented as an array of moves.
/// - deadline: A timestamp indicating the deadline for finalizing the game session.
/// - close_game_session: A boolean indicating whether to close the game session account
///   after finalization. If true, the account will be closed and remaining lamports will
///   be transferred to the player. If false, the account will remain open with updated
///   state.
#[derive(AnchorDeserialize, AnchorSerialize, Clone, Default)]
pub struct FinalizeLossArgs {
    pub private_config_seed:[u8;HASH_LENGTH],
    pub finalized_game_state:[u8;MAX_MOVE_COUNT],
    pub deadline:i64,
    pub close_game_session:bool
}

#[derive(Accounts)]
#[instruction(args: FinalizeLossArgs)]
/// Accounts for finalizing a game session as a loss.
/// - game_session: The game session account to be finalized.
/// - player: The player who owns the game session. This account must sign the
///   transaction.
/// - global_state: The global state account containing the game administrator's
///   information and the vault address.
pub struct FinalizeLossCtx<'info> {
    #[account(
        mut
    )]
    pub game_session: Account<'info, GameSession>, 

    #[account(
        mut
    )]
    pub player: Signer<'info>,

    #[account(
        seeds = [b"global-state"],
        bump = global_state.get_bump(),
    )]
    pub global_state: Account<'info, GlobalState>,

    pub system_program: Program<'info, System>,

    /// CHECK: This is the instruction sysvar account
    #[account(
        address = INSTRUCTIONS_SYSVAR_ADDRESS
    )]
    pub instructions_sysvar: UncheckedAccount<'info>
}

#[inline(always)] // This function is only called once, in the handler.
/// Perform the preliminary checks, other checks may be perfomed later in the handler.
pub fn checks(
    ctx: &Context<FinalizeLossCtx>,
    args:&FinalizeLossArgs,
)->Result<()>{

    let current_timestamp = Clock::get()?.unix_timestamp;

    // Verify that the game session is active
    require!(
        ctx.accounts.game_session.get_status() == crate::GameStatus::Active,
        GameError::GameNotActive
    );

    // Verify we are not past the deadline for the signature
    require_gt!(
        args.deadline,
        current_timestamp,
        GameError::DeadlinePassed    
    );

    // Verify the player is the owner of the game session.
    require_keys_eq!(
        *ctx.accounts.player.key,
        *ctx.accounts.game_session.get_player(),
        GameError::InvalidPlayer
    );

    {// IMPORTANT: Do not change the commitment without taking into consideration the fact that
    // the message could be manipulated, if the field lengths are variable, in that case length
    // prefixes would need to be added, the fields are fixed here so they are left as is.

    let deadline = args.deadline.to_le_bytes();

    // Build an array of references to the data slices that make up the commitment message.
    let commitment = [
        FINALIZE_LOSS_ACTION.as_bytes(),
        &deadline,
        &args.finalized_game_state,
        // The commitment commits to the game's public and private configuration seeds which
        // are for example used to derive the tile counts and the death tile positions, so
        // they are all implictly included in the commitment.
        // It is also tied to the session as the session's key is derived from it, so it 
        // cannot be reused for sessions.            
        ctx.accounts.game_session.get_commitment().as_ref(),
    ];

    // Verify the ED25519 signature is valid.
    is_signature_valid(
        &ctx.accounts.instructions_sysvar.to_account_info(),
        &commitment,
        &ctx.accounts.global_state.message_signer
    )}
}

pub fn finalize_loss_handler(
    ctx:Context<FinalizeLossCtx>,
    args:FinalizeLossArgs
)->Result<()>{

    // Perform the preliminary checks.
    checks(&ctx, &args)?;

    let game_session = &mut ctx.accounts.game_session;

    // If requested, close the game session account and transfer remaining lamports to the player.
    if args.close_game_session {
        game_session.close(ctx.accounts.player.to_account_info())?;
    } else{
        // Update the game session state to reflect the finalized loss only if the user 
        // wants to persist the account.
        game_session.set_private_config_seed(args.private_config_seed);
        game_session.set_finalized_game_state(args.finalized_game_state);
        game_session.set_status(GameStatus::Lost)?;
    }

    Ok(())
}

#[cfg(test)]
mod tests{
    use super::*;

    pub const fn is_sized_type<T:Sized + Copy>(_:&T){}

    #[test]
    pub fn test_hash_args(){
        let dummy_arg = FinalizeLossArgs::default();

        is_sized_type(&dummy_arg.deadline);
        is_sized_type(&dummy_arg.finalized_game_state);
    }
}