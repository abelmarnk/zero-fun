use anchor_lang::{
    prelude::*,
    solana_program::hash::hashv
};

use crate::{
    FinalizeGameAsLostEvent, GameError, GameSession, GlobalState, HASH_LENGTH, MAX_MOVE_TYPE_COUNT, PUBLIC_SEED
};

/// Arguments for finalizing a game session as a loss.
/// - private_config_seed: The SHA-256 hash seed used to derive the private configuration
///   of the game.
/// - fail_position: The position that resulted in failure for the player
#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct FinalizeGameAsLostArgs {
    pub private_config_seed:[u8;HASH_LENGTH],
    pub fail_position:u8,
}

#[derive(Accounts)]
pub struct FinalizeGameAsLostAccounts<'info> {
    #[account(
        mut,
        close = player
    )]
    pub game_session: Account<'info, GameSession>, 

    #[account(
        mut
    )]
    pub player: Signer<'info>,

    /// CHECK: This is the vault account where the player's deposit is stored.
    #[account(
        mut
    )]
    pub user_vault: UncheckedAccount<'info>,

    /// CHECK: This is the global vault account.
    #[account(
        mut,
        seeds = [b"vault"],
        bump = global_state.get_vault_bump()
    )]
    pub vault: UncheckedAccount<'info>,

    pub global_state: Account<'info, GlobalState>,
}

#[inline(always)]
fn checks(
    ctx: &Context<FinalizeGameAsLostAccounts>,
    args:&FinalizeGameAsLostArgs,
)->Result<()>{

    require!(
        ctx.accounts.game_session.is_active(),
        GameError::GameSessionNotActive
    );

    require!(
        ctx.accounts.game_session.is_owned_by_player(ctx.accounts.player.key),
        GameError::InvalidPlayer
    );

    require!(
        ctx.accounts.game_session.is_vault_for_game(ctx.accounts.user_vault.key),
        GameError::InvalidVault
    );

    // Verify the public config was previously commited to.
    let public_config_seed:[u8;HASH_LENGTH] = hashv(&[
            PUBLIC_SEED.as_ref(),
            args.private_config_seed.as_ref()
        ]).to_bytes();

    require!(
        ctx.accounts.game_session.public_config_seed.eq(&public_config_seed),
        GameError::InvalidGameSeed
    );

    // Get the number of moves for this round
    let public_config_seed_for_move:[u8;HASH_LENGTH] = hashv(&[
            &[args.fail_position],
            ctx.accounts.game_session.public_config_seed.as_ref()
        ]).to_bytes();

    let move_type_count_for_round = (public_config_seed_for_move[0] % 
        u8::try_from(MAX_MOVE_TYPE_COUNT).unwrap() - 1) + 2;

    // Get the move for failure
    let private_config_seed_for_move:[u8;HASH_LENGTH] = hashv(&[
            &[args.fail_position],
            args.private_config_seed.as_ref()
        ]).to_bytes();    

    let fail_move = private_config_seed_for_move[0] % move_type_count_for_round;

    // Verify the player made that move
    require!(
        ctx.accounts.game_session.next_player_move_position.gt(&args.fail_position) &&
        ctx.accounts.game_session.player_moves[usize::from(args.fail_position)].eq(&fail_move),
        GameError::InvalidFailPosition
    );
    Ok(())
}

pub fn finalize_game_as_lost_handler(
    ctx:Context<FinalizeGameAsLostAccounts>,
    args:FinalizeGameAsLostArgs
)->Result<()>{

    checks(&ctx, &args)?;

    // Transfer funds to the main vault
    **ctx.accounts.vault.try_borrow_mut_lamports()? += ctx.accounts.user_vault.lamports();
    **ctx.accounts.user_vault.try_borrow_mut_lamports()? = 0;

    emit!(
        FinalizeGameAsLostEvent{
            game_session:ctx.accounts.game_session.key(),
            private_config_seed:args.private_config_seed
        }
    );
    Ok(())
}
