use anchor_lang::{
    prelude::*, solana_program::sysvar::instructions::ID as INSTRUCTIONS_SYSVAR_ADDRESS, 
    system_program::{
        Transfer,
        transfer
    }
};

use crate::{
    GameError, GameSession, GlobalState, HASH_LENGTH, INITIALIZE_GAME_ACTION, 
    MAX_BPS, MAX_METADATA_LENGTH, is_signature_valid
};

/// Arguments for initializing a new game session.
/// - commitment: A SHA-256 hash commitment to all the games relevant configuration.
/// - public_config_seed: A SHA-256 hash seed used to derive the public configuration of
///   the game.
/// - game_metadata: Arbitrary metadata about the game, such as the algorithm version,
///   configuration parameters, etc. Limited to 64 bytes.
/// - deposit: The amount of lamports the player is depositing to play the game.
/// - admin_signature: A signature from the game administrator authorizing the game
///   initialization. This is used to prevent unauthorized game sessions.
#[derive(AnchorDeserialize, AnchorSerialize, Clone, Default)]
pub struct InitializeGameArgs {
    pub commitment: [u8; HASH_LENGTH],
    pub public_config_seed: [u8; HASH_LENGTH],
    pub game_metadata: String,
    pub deposit: u64,
    pub deadline:i64
}

/// Accounts for initializing a new game session.
/// - game_session: The account to store the game session data. This account is initialized
///   in this instruction.
/// - player: The player initializing the game session. This account must sign the
///   transaction.
/// - vault: The vault account where the player's deposit will be stored. This account
///   must match the vault specified in the global state.
/// - global_state: The global state account containing the game administrator's
///   information and the vault address.
#[derive(Accounts)]
#[instruction(args: InitializeGameArgs)]
pub struct InitializeGameCtx<'info> {
    #[account(
        init, 
        payer = player, 
        space = 8 + GameSession::INIT_SPACE,
        // The seeds are expected to be unique for each game session because it is a commitment to
        // the both the private and public configuration which were derived from a random seed.
        seeds = [b"game-session".as_ref(), args.commitment.as_ref(), {player.key().as_ref()}],
        bump
    )]
    pub game_session: Account<'info, GameSession>,

    #[account(
        mut
    )]
    pub player: Signer<'info>,

    /// CHECK: This is the vault account where the player's deposit will be stored.
    #[account(
        mut,
        seeds = [b"vault"],
        bump
    )]
    pub vault: UncheckedAccount<'info>,

    #[account(
        seeds = [b"global-state"],
        bump = global_state.get_bump()
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
fn checks(
    ctx: &Context<InitializeGameCtx>,
    args: &InitializeGameArgs,
    current_timestamp:i64
)-> Result<()>{

    // Verify we are not past the deadline for the signature
    require_gt!(
        args.deadline,
        current_timestamp,
        GameError::DeadlinePassed    
    );

    // Verify the game metadata length is within bounds.
    require_gte!(
        MAX_METADATA_LENGTH,
        args.game_metadata.len() ,
        GameError::MetadataTooLong
    );

    // Verify the deposit is within the allowed maximum deposit.
    let current_max_deposit = ctx.accounts.vault.lamports().
        checked_mul(u64::from(ctx.accounts.global_state.max_deposit)).
        ok_or(ProgramError::ArithmeticOverflow)?.saturating_div(MAX_BPS);

    require_gt!(
        current_max_deposit,
        args.deposit,
        GameError::DepositExceedsMaximum
    );
    
    // IMPORTANT: Do not change the commitment without taking into consideration the fact that
    // the message could be manipulated, if the field lengths are variable, in that case length
    // prefixes would need to be added, there is only one variable length field 
    // here so they are left as is.
    {let deposit = args.deposit.to_le_bytes();
    let deadline = args.deadline.to_le_bytes();

    // Build an array of references to the data slices that make up the commitment message.
    let commitment = [
        INITIALIZE_GAME_ACTION.as_bytes(),
        // The commitment commits to the game's public and private configuration seeds which
        // are for example used to derive the tile counts and the death tile positions, so
        // they are all implictly included in the commitment.
        // It is also tied to the session as the session's key is derived from it, so it 
        // cannot be reused for sessions.
        &args.commitment,
        &deposit,
        &deadline,
        args.game_metadata.as_bytes(),
        ctx.accounts.player.key.as_array().as_ref(),
    ];

    // Verify the admin's signature on the commitment message.
    is_signature_valid(
        &ctx.accounts.instructions_sysvar.to_account_info(),
        &commitment,
        &ctx.accounts.global_state.message_signer,
    )}
}

pub fn initialize_game_handler(
    ctx: Context<InitializeGameCtx>,
    args: InitializeGameArgs,
) -> Result<()> {

    // Get the current timestamp.
    let curent_timestamp = Clock::get()?.unix_timestamp;
    
    // Perform necessary checks before initializing the game session.
    checks(&ctx, &args, curent_timestamp)?;
    
    let game_session = &mut ctx.accounts.game_session;

    // Initialize the game session account with the provided arguments.
    game_session.set_inner(GameSession::new(
        ctx.accounts.player.key(),
        args.deposit,
        args.commitment,
        args.public_config_seed,
        args.game_metadata,
        curent_timestamp
    ));

    // Transfer the player's deposit to the vault.
    transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer{
                from: ctx.accounts.player.to_account_info(),
                to: ctx.accounts.vault.to_account_info()
            }
        ),
        args.deposit
    )?;

    Ok(())
}


#[cfg(test)]
mod tests{
    use super::*;

    pub const fn is_sized_type<T:Sized + Copy>(_:&T){}

    #[test]
    pub fn test_hash_args(){
        let dummy_arg = InitializeGameArgs::default();

        is_sized_type(&dummy_arg.commitment);
        is_sized_type(&dummy_arg.deadline);
        is_sized_type(&dummy_arg.deposit);
        is_sized_type(&dummy_arg.public_config_seed);
    }
}