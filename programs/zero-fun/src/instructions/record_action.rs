use anchor_lang::prelude::*;

use crate::{GameError, GameSession, GlobalState};

#[derive(AnchorDeserialize, AnchorSerialize, Clone, Default)]
pub struct RecordActionArgs{
    pub action:u8
}

#[derive(Accounts)]
pub struct RecordActionAccounts<'info>{
    player:Signer<'info>,

    global_state:Account<'info, GlobalState>,

    game_session:Account<'info, GameSession>,
}

#[inline(always)]
fn checks(ctx:&Context<RecordActionAccounts>)->Result<()>{

    require!(
        ctx.accounts.game_session.is_owned_by_player(ctx.accounts.player.key),
        GameError::InvalidPlayer
    );

    require!(
        ctx.accounts.game_session.is_active(),
        GameError::GameSessionNotActive
    );

    require!(
        ctx.accounts.global_state.is_active(),
        GameError::GameNotActive
    );

    Ok(())
}


pub fn record_action_handler(ctx:Context<RecordActionAccounts>, args:RecordActionArgs)->Result<()>{
    checks(&ctx)?;

    // Update the last action time
    let now = Clock::get()?.unix_timestamp;
    ctx.accounts.game_session.last_action_time = now;

    // Record the player's move
    ctx.accounts.game_session.set_next_player_move(args.action)?;

    Ok(())
}