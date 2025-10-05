use anchor_lang::{
    prelude::*,
    solana_program::sysvar::instructions::ID as INSTRUCTIONS_SYSVAR_ADDRESS
};

use crate::{
    FINALIZE_WIN_ACTION, FinalizeGameAsWonEvent, GameError, GameSession, GlobalState, HASH_LENGTH, MAX_BPS, is_signature_valid
};

/// Arguments for finalizing a game session as a win.
/// - payout: The amount of lamports to be paid out to the player upon winning the game.
/// - private_config_seed: The SHA-256 hash seed used to derive the private configuration
///   of the game.
/// - deadline: A timestamp indicating the deadline for the signature provided.
#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct FinalizeGameAsWonArgs {
    pub payout:u64,
    pub deadline:i64
}

#[derive(Accounts)]
#[instruction(args: FinalizeGameAsWonArgs)]
pub struct FinalizeGameAsWonAccounts<'info> {
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

    pub system_program: Program<'info, System>,

    /// CHECK: This is the instruction sysvar account    
    #[account(
        address = INSTRUCTIONS_SYSVAR_ADDRESS
    )]
    pub instructions_sysvar: UncheckedAccount<'info>
}

#[inline(always)]
fn checks(
    ctx: &Context<FinalizeGameAsWonAccounts>,
    args:&FinalizeGameAsWonArgs,
)->Result<()>{
    
    require!(
        ctx.accounts.game_session.is_active(),
        GameError::GameSessionNotActive
    );

    let now = Clock::get()?.unix_timestamp;

    require_gt!(
        args.deadline,
        now,
        GameError::DeadlinePassed    
    );

    require!(
        ctx.accounts.game_session.is_owned_by_player(ctx.accounts.player.key),
        GameError::InvalidPlayer
    );

    require!(
        ctx.accounts.game_session.is_vault_for_game(ctx.accounts.user_vault.key),
        GameError::InvalidVault
    );
    

    // Verify the payout does not exceed the maximum allowed payout.
    let current_max_payout = ctx.accounts.vault.lamports().
        checked_mul(u64::from(ctx.accounts.global_state.max_payout)).
        ok_or(ProgramError::ArithmeticOverflow)?/MAX_BPS;

    require_gt!(
        current_max_payout,
        args.payout,
        GameError::PayoutExceedsMaximum
    );

    // IMPORTANT: Do not change the commitment without taking into consideration the fact that
    // the message could be manipulated if the field lengths are variable, in that case length
    // prefixes would need to be added, the fields are fixed here so they are left as is.
    {let deadline = args.deadline.to_le_bytes();
    let payout = args.payout.to_le_bytes();

    // Build an array of references to the data slices that make up the commitment message.
    let commitment = [
        FINALIZE_WIN_ACTION.as_bytes(),
        &payout,
        &deadline,
        // The commitment commits to the game's public and private configuration seeds which
        // are for example used to derive the tile counts and the death tile positions, so
        // they are all implictly included in the commitment.
        // It is also tied to the session as the session's key is derived from it, so it 
        // cannot be reused for sessions.
        ctx.accounts.game_session.public_config_seed.as_ref(),
    ];

    // Verify the ED25519 signature is valid.
    is_signature_valid(
        &ctx.accounts.instructions_sysvar.to_account_info(),
        &commitment,
        &ctx.accounts.global_state.message_signer
    )}
}

pub fn finalize_game_as_won_handler(
    ctx:Context<FinalizeGameAsWonAccounts>,
    args:FinalizeGameAsWonArgs
)->Result<()>{

    checks(&ctx, &args)?;

    // Transfer the winnings to the player
    **ctx.accounts.player.try_borrow_mut_lamports()? += 
    ctx.accounts.user_vault.lamports() + args.payout;

    // The vault has had it's lamports(both the deposit and rent) transferred 
    // back to the user
    **ctx.accounts.user_vault.try_borrow_mut_lamports()? = 0;

    // Deduct the payout from the global vault
    **ctx.accounts.vault.try_borrow_mut_lamports()? -= args.payout;

    emit!(
        FinalizeGameAsWonEvent{
            payout:args.payout,
            game_session:ctx.accounts.game_session.key()
        }
    );

    Ok(())
}
