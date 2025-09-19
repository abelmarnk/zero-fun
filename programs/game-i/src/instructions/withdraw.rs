use anchor_lang::prelude::*;

use crate::GlobalState;


#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct WithdrawArgs {
    pub amount: u64,
}

#[derive(Accounts)]
pub struct WithdrawCtx<'info> {
    #[account(
        seeds = [b"global-state"],
        bump = global_state.get_bump(),
    )]
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        seeds = [b"vault"],
        bump
    )]
    /// CHECK: Vault account from which funds will be withdrawn
    pub vault: UncheckedAccount<'info>,

    #[account(
        mut
    )]
    /// CHECK: Vault recipient account to receive the withdrawn funds
    pub recipient: UncheckedAccount<'info>,

    /// The admin must sign to authorize the withdrawal.
    pub admin: Signer<'info>,

    pub system_program: Program<'info, System>,
}


#[inline(always)] // This function is only called once, in the handler.
/// Perform the preliminary checks, other checks may be perfomed later in the handler.
fn checks(
    ctx: &Context<WithdrawCtx>
) -> Result<()> {
    // Only the current admin can authorize withdrawals.
    require_keys_eq!(
        ctx.accounts.admin.key(),
        ctx.accounts.global_state.admin,
        crate::GameError::InvalidAdmin
    );


    Ok(())
}

/// Handler for withdrawing funds from the vault.
pub fn withdraw_handler(
    ctx: Context<WithdrawCtx>,
    args: WithdrawArgs,
) -> Result<()> {
    checks(&ctx)?;

    **ctx.accounts.vault.to_account_info().try_borrow_mut_lamports()? -= args.amount;
    **ctx.accounts.recipient.to_account_info().try_borrow_mut_lamports()? += args.amount;

    Ok(())
}