use anchor_lang::{prelude::*};

use crate::{DefaultGameEvent, GameError, GameSession};


#[derive(Accounts)]
pub struct DefaultGameAccounts<'info>{
    player:Signer<'info>,

    #[account(
        mut
    )]
    user_vault:UncheckedAccount<'info>,

    #[account(
        mut,
        close = player
    )]
    game_session:Account<'info, GameSession>,
}

#[inline(always)]
fn checks(ctx:&Context<DefaultGameAccounts>)->Result<()>{

    require!(
        ctx.accounts.game_session.is_vault_for_game(ctx.accounts.user_vault.key),
        GameError::InvalidVault
    );
    
    require!(
        ctx.accounts.game_session.is_owned_by_player(ctx.accounts.player.key),
        GameError::InvalidPlayer
    );

    let now = Clock::get()?.unix_timestamp; 

    require!(
        ctx.accounts.game_session.can_default(now),
        GameError::TooSoonToDefault    
    );

    Ok(())
}


pub fn default_game_handler(ctx:Context<DefaultGameAccounts>)->Result<()>{

    checks(&ctx)?;
    
    // Transfer the player's deposit back.
    **ctx.accounts.player.try_borrow_mut_lamports()? += ctx.accounts.user_vault.lamports();

    **ctx.accounts.user_vault.try_borrow_mut_lamports()? = 0;

    emit!(
        DefaultGameEvent{
            game_session:ctx.accounts.game_session.key()
        }
    );
    Ok(())
}