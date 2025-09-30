use anchor_lang::prelude::*;
use crate::{GlobalState, WithdrawEvent};

#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct WithdrawArgs {
    pub amount: u64,
}

#[derive(Accounts)]
pub struct WithdrawAccounts<'info> {
    pub global_state: Account<'info, GlobalState>,

    #[account(
        mut,
        seeds = [b"vault"],
        bump = global_state.get_vault_bump()
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


#[inline(always)]
fn checks(
    ctx: &Context<WithdrawAccounts>
) -> Result<()> {
    require!(
        ctx.accounts.global_state.is_admin(ctx.accounts.admin.key),
        crate::GameError::InvalidAdmin
    );

    Ok(())
}

pub fn withdraw_handler(
    ctx: Context<WithdrawAccounts>,
    args: WithdrawArgs,
) -> Result<()> {
    checks(&ctx)?;

    **ctx.accounts.vault.to_account_info().try_borrow_mut_lamports()? -= args.amount;
    **ctx.accounts.recipient.to_account_info().try_borrow_mut_lamports()? += args.amount;

    emit!(
        WithdrawEvent{
            admin:ctx.accounts.admin.key(),
            recipient:ctx.accounts.recipient.key(),
            amount:args.amount
        }
    );

    Ok(())
}