use anchor_lang::{
    prelude::*,
    solana_program::sysvar::instructions::ID as INSTRUCTIONS_SYSVAR_ADDRESS
};

use crate::{
    FinalizeGameAsWonForPlayerEvent, GameError, GameSession, GlobalState
};


#[derive(AnchorDeserialize, AnchorSerialize, Clone)]
pub struct FinalizeGameAsWonForPlayerArgs {
    pub payout:u64
}

#[derive(Accounts)]
pub struct FinalizeGameAsWonForPlayerAccounts<'info> {
    #[account(
        mut,
        // In return for closing the game for the user the admin would 
        // get the rent for the vault & game session
        close = vault
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

    pub admin:Signer<'info>,

    pub system_program: Program<'info, System>,

    /// CHECK: This is the instruction sysvar account    
    #[account(
        address = INSTRUCTIONS_SYSVAR_ADDRESS
    )]
    pub instructions_sysvar: UncheckedAccount<'info>
}

#[inline(always)]
fn checks(
    ctx: &Context<FinalizeGameAsWonForPlayerAccounts>
)->Result<()>{

    // Verify that the game session has been marked as won by the player
    require!(
        ctx.accounts.game_session.is_won(),
        GameError::GameSessionNotWon
    );

    require!(
        ctx.accounts.game_session.is_owned_by_player(ctx.accounts.player.key),
        GameError::InvalidPlayer
    );

    require!(
        ctx.accounts.game_session.is_vault_for_game(ctx.accounts.user_vault.key),
        GameError::InvalidVault
    );

    require!(
        ctx.accounts.global_state.is_admin(ctx.accounts.admin.key),
        GameError::InvalidAdmin
    );

    Ok(())
}

pub fn finalize_game_as_won_for_player_handler(
    ctx:Context<FinalizeGameAsWonForPlayerAccounts>,
    args:FinalizeGameAsWonForPlayerArgs
)->Result<()>{

    checks(&ctx)?;

    
    // Transfer the winnings to the player
    **ctx.accounts.player.try_borrow_mut_lamports()? += 
    ctx.accounts.game_session.deposit + args.payout;
    
    let rent_exempt_fee = ctx.accounts.user_vault.lamports() - ctx.accounts.game_session.deposit;
    
    // The vault has had it's lamports(both the deposit and rent) transferred 
    // back to the user and global vault
    **ctx.accounts.user_vault.try_borrow_mut_lamports()? = 0;
    
    // In return for closing the game for the user the admin would get 
    // the rent for the vault & game session, that is deducted from the payout
    **ctx.accounts.vault.try_borrow_mut_lamports()? -= args.payout - rent_exempt_fee;

    emit!(
        FinalizeGameAsWonForPlayerEvent{
            admin:ctx.accounts.admin.key(),
            payout:args.payout,
            game_session:ctx.accounts.game_session.key()
        }
    );

    Ok(())
}
