use anchor_lang::prelude::*;

use crate::GlobalState;

/// This key should be replaced with an actual key we control, before deployment.
/// it is delibrately left empty here to avoid accidental usage, the complier will complain
/// if it is not replaced.
/// It is only to be used once, to initialize the global state.
const BOOTSTRAP_KEY:Pubkey = pubkey!("5e4vTmm5pcUFHPr34rtrpu33kXC5nG4eN7JmkHhJpJsP");

/// Arguments for initializing the global state.
/// - admin: The admin's public key.
/// - message_signer: The public key used to verify messages.
/// - vault: The vault account public key.
/// - max_deposit: Maximum deposit allowed (in bps).
/// - max_payout: Maximum payout allowed (in bps).
#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct InitializeGlobalStateArgs {
    pub admin: Pubkey,
    pub message_signer: Pubkey,
    pub vault: Pubkey,
    pub max_deposit: u8,
    pub max_payout: u8,
}

#[derive(Accounts)]
#[instruction(args: InitializeGlobalStateArgs)]
pub struct InitializeGlobalStateCtx<'info> {
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

    /// CHECK: The vault account where deposits are stored.
    pub vault: UncheckedAccount<'info>,

    pub system_program: Program<'info, System>,
}

#[inline(always)] // This function is only called once, in the handler.
/// Perform preliminary checks, other checks may be performed later in the handler.
pub fn checks(
    ctx: &Context<InitializeGlobalStateCtx>,
    args: &InitializeGlobalStateArgs
)->Result<()>{
    // Ensure the initializer is the bootstrap key to prevent unauthorized initialization.
    require_keys_eq!(
        ctx.accounts.initializer.key(),
        BOOTSTRAP_KEY,
        crate::GameError::InvalidBootstrapKey
    );

    // Ensure that the admin key is not the same as the bootstrap key.
    require_keys_neq!(
        args.admin,
        BOOTSTRAP_KEY,
        crate::GameError::InvalidAdmin
    );

    Ok(())
}


/// Handler for initializing the global state.
pub fn initialize_global_state_handler(
    ctx: Context<InitializeGlobalStateCtx>,
    args: InitializeGlobalStateArgs
) -> Result<()> {

    checks(&ctx, &args)?;

    let global_state = &mut ctx.accounts.global_state;

    global_state.set_inner(GlobalState::new(
        args.admin,
        args.message_signer,
        args.vault,
        args.max_deposit,
        args.max_payout,
        ctx.bumps.global_state,
    ));

    Ok(())
}