use anchor_lang::{
    prelude::*,
    system_program::{
        Transfer,
        transfer
    }
};

use crate::{
    GameError, GameSession, GlobalState, HASH_LENGTH, InitializeGameEvent, MAX_BPS, MAX_METADATA_LENGTH
};

/// Arguments for initializing a new game session.
/// - public_config_seed: A SHA-256 hash seed used to derive the public configuration of
///   the game.
/// - game_metadata: Arbitrary metadata about the game, such as the algorithm version,
///   configuration parameters, etc. Limited to 64 bytes.
#[derive(AnchorDeserialize, AnchorSerialize, Clone, Default)]
pub struct InitializeGameArgs {
    pub public_config_seed: [u8; HASH_LENGTH],
    pub game_metadata: String,
    pub deposit: u64,
}


#[derive(Accounts)]
#[instruction(args: InitializeGameArgs)]
pub struct InitializeGameAccounts<'info> {
    #[account(
        init, 
        payer = player, 
        space = 8 + GameSession::INIT_SPACE,
        // The seeds are expected to be unique for each game session because it is a commitment to
        // the both the public configuration which was derived from a random seed.
        seeds = [b"game-session".as_ref(), args.public_config_seed.as_ref(), {player.key().as_ref()}],
        bump
    )]
    pub game_session: Account<'info, GameSession>,

    #[account(
        mut
    )]
    pub player: Signer<'info>,

    /// CHECK: This is the vault account where the player's deposit will be stored.
    #[account(
        init,
        space = 0,
        payer = player,
        seeds = [b"vault", args.public_config_seed.as_ref(), player.key.as_ref()],
        bump
    )]
    pub user_vault: UncheckedAccount<'info>,

    /// CHECK: This is the global vault account.
    #[account(
        seeds = [b"vault"],
        bump = global_state.get_vault_bump()
    )]
    pub vault: UncheckedAccount<'info>,

    pub global_state: Account<'info, GlobalState>,

    pub system_program: Program<'info, System>,
}

#[inline(always)]
fn checks(
    ctx: &Context<InitializeGameAccounts>,
    args: &InitializeGameArgs,
)-> Result<()>{

    // Verify the game metadata length is within bounds.
    require_gte!(
        MAX_METADATA_LENGTH,
        args.game_metadata.len() ,
        GameError::MetadataTooLong
    );

    // Verify the deposit is within the allowed maximum deposit.
    let current_max_deposit = ctx.accounts.vault.lamports().
        checked_mul(u64::from(ctx.accounts.global_state.max_deposit)).
        ok_or(ProgramError::ArithmeticOverflow)?/MAX_BPS;

    require_gte!(
        current_max_deposit,
        args.deposit,
        GameError::DepositExceedsMaximum
    );

    // Verfiy that the game is still active
    require!(
        ctx.accounts.global_state.is_active(),
        GameError::GameNotActive
    );

    Ok(())
}

pub fn initialize_game_handler(
    ctx: Context<InitializeGameAccounts>,
    args: InitializeGameArgs,
) -> Result<()> {
    
    checks(&ctx, &args)?;
    
    let game_session = &mut ctx.accounts.game_session;

    let now = Clock::get()?.unix_timestamp;

    game_session.set_inner(GameSession::new(
        ctx.accounts.player.key(),
        args.deposit,
        *ctx.accounts.vault.key,
        args.public_config_seed,
        args.game_metadata,
        now,
    ));

    transfer(
        CpiContext::new(
            ctx.accounts.system_program.to_account_info(),
            Transfer{
                from: ctx.accounts.player.to_account_info(),
                to: ctx.accounts.user_vault.to_account_info()
            }
        ),
        args.deposit
    )?;


    emit!(
        InitializeGameEvent{
            game_session:ctx.accounts.game_session.key(),
            game_session_account:(*ctx.accounts.game_session).clone()
        }
    );

    Ok(())
}
