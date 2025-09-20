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
    pub max_deposit: u8,
    pub max_payout: u8
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

    #[account(
        init,
        space = 0,
        payer = initializer,
        seeds = [b"vault"],
        bump
    )]
    pub vault: UncheckedAccount<'info>,

    /// This is added as a signer to guarantee the account is controlled by them
    pub message_signer: Signer<'info>,

    /// This is added as a signer to guarantee the account is controlled by them    
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}

#[inline(always)] // This function is only called once, in the handler.
/// Perform preliminary checks, other checks may be performed later in the handler.
fn checks(
    ctx: &Context<InitializeGlobalStateCtx>
)->Result<()>{
    // Ensure the initializer is the bootstrap key to prevent unauthorized initialization.
    require_keys_eq!(
        ctx.accounts.initializer.key(),
        BOOTSTRAP_KEY,
        crate::GameError::InvalidBootstrapKey
    );

    // Ensure that the admin key is not the same as the bootstrap key.
    require_keys_neq!(
        *ctx.accounts.admin.key,
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

    checks(&ctx)?;

    let global_state = &mut ctx.accounts.global_state;

    global_state.set_inner(GlobalState::new(
        *ctx.accounts.admin.key,
        *ctx.accounts.message_signer.key,
        args.max_deposit,
        args.max_payout,
        ctx.bumps.global_state,
    ));

    Ok(())
}