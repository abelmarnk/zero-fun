use anchor_lang::prelude::*;
use crate::{GameState, GlobalState};


const INITIALIZER_KEY:Pubkey = pubkey!("4w5ezXcjV8RdJLPAQmwVonevgUVfuAZSDMdWtURc1CRY");

/// Arguments for initializing the global state.
/// - max_deposit: Maximum deposit allowed (in bps).
/// - max_payout: Maximum payout allowed (in bps).
/// - initial_state: The initial state the game is in, 
/// it can be changed later.
#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct InitializeGlobalStateArgs {
    pub max_deposit: u8,
    pub max_payout: u8,
    pub initial_state:GameState
}

#[derive(Accounts)]
#[instruction(args: InitializeGlobalStateArgs)]
pub struct InitializeGlobalStateAccounts<'info> {
    #[account(
        init,
        payer = initializer,
        space = 8 + GlobalState::INIT_SPACE,
        seeds = [b"global-state"],
        bump
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut
    )]
    pub initializer: Signer<'info>,

    #[account(
        init,
        space = 0,
        payer = initializer,
        seeds = [b"vault"],
        bump
    )]
    pub vault: UncheckedAccount<'info>,

    // This is added as a signer to guarantee the account is controlled by them
    pub message_signer: Signer<'info>,

    // This is added as a signer to guarantee the account is controlled by them    
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[inline(always)]
fn checks(
    ctx: &Context<InitializeGlobalStateAccounts>
)->Result<()>{
    // Ensure the initializer is the bootstrap key
    require_keys_eq!(
        ctx.accounts.initializer.key(),
        INITIALIZER_KEY,
        crate::GameError::InvalidBootstrapKey
    );

    Ok(())
}


pub fn initialize_global_state_handler(
    ctx: Context<InitializeGlobalStateAccounts>,
    args: InitializeGlobalStateArgs
) -> Result<()> {

    checks(&ctx)?;

    let global_state = &mut ctx.accounts.global_state;

    global_state.set_inner(GlobalState::new(
        *ctx.accounts.admin.key,
        *ctx.accounts.message_signer.key,
        args.max_deposit,
        args.max_payout,
        args.initial_state,
        ctx.bumps.vault,
    ));

    Ok(())
}