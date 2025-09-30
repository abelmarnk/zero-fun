use anchor_lang::prelude::*;

use crate::{GameError, GameSession, GameSessionStatus, GlobalState, MarkGameAsWonEvent};


#[derive(Accounts)]
pub struct MarkGameAsWonAccounts<'info>{
    player:Signer<'info>,

    global_state:Account<'info, GlobalState>,

    game_session:Account<'info, GameSession>,
}

#[inline(always)]
fn checks(ctx:&Context<MarkGameAsWonAccounts>)->Result<()>{

    require!(
        ctx.accounts.game_session.is_owned_by_player(ctx.accounts.player.key),
        GameError::InvalidPlayer
    );

    require!(
        ctx.accounts.game_session.is_active(),
        GameError::GameSessionNotActive
    );

    Ok(())
}

pub fn mark_game_as_won_handler(ctx:Context<MarkGameAsWonAccounts>)->Result<()>{
    checks(&ctx)?;

    ctx.accounts.game_session.status = GameSessionStatus::Won;

    emit!(
        MarkGameAsWonEvent{
            game_session:ctx.accounts.game_session.key()
        }
    );

    Ok(())
}