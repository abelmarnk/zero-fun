use anchor_lang::InstructionData;
use litesvm::LiteSVM;
use anyhow::Result;
use solana_sdk::{
    instruction::{AccountMeta, Instruction}, 
    pubkey::Pubkey, signer::{Signer, keypair::Keypair}, 
    transaction::Transaction
};

mod common;
use common::utils::{
    add_zero_fun_program,
    create_game_session_account,
    create_global_state_account,
};

use zero_fun::{
    instruction::{RecordAction},
    instructions::RecordActionArgs,
    GameSession,
    GameSessionStatus,
    GlobalState,
    GameState,
    HASH_LENGTH,
    ID as ZERO_FUN_PROGRAM_ID,
};

struct TestSetup {}

impl TestSetup {
    const ZERO_FUN_PROGRAM_ID: Pubkey = ZERO_FUN_PROGRAM_ID;

    fn builder(
        svm: &mut LiteSVM,
        state_player: Pubkey,
        instruction_player: Keypair,
        global_state_status: GameState,
        game_session_status: GameSessionStatus,
    ) -> Result<([Instruction; 1], Vec<Keypair>)> {

        // Create the player account
        svm.airdrop(&instruction_player.pubkey(), 1_000_000_000)
            .expect("Could not airdrop to player");

        // Create the Global State
        let global_state_pubkey = Pubkey::new_unique();
        let global_state = GlobalState::new(
            Pubkey::new_unique(),             
            Pubkey::new_unique(),    
            10,                         
            100,                          
            global_state_status,         
            255,                         
        );

        create_global_state_account(svm, global_state_pubkey, global_state);

        // Create the Game Session
        let (game_session_pubkey, _) = Pubkey::find_program_address(
            &[
                b"game-session",
                [0u8; HASH_LENGTH].as_ref(),
                state_player.as_ref(),
            ],
            &Self::ZERO_FUN_PROGRAM_ID,
        );

        let game_session_account = GameSession {
            last_action_time: 0,
            player: state_player,
            deposit: 1_000_000u64,
            status: game_session_status,
            public_config_seed: [0u8; HASH_LENGTH],
            game_metadata: "metadata".to_string(),
            player_moves: [0u8; zero_fun::MAX_MOVE_COUNT],
            vault: Pubkey::new_unique(),
            next_player_move_position: 0u8,
        };

        create_game_session_account(svm, game_session_pubkey, &game_session_account);

        // Build the instruction
        let accounts: Vec<AccountMeta> = vec![
            AccountMeta::new(instruction_player.pubkey(), true),
            AccountMeta::new(global_state_pubkey, false),
            AccountMeta::new(game_session_pubkey, false),
        ];

        let instruction = Instruction {
            program_id: Self::ZERO_FUN_PROGRAM_ID,
            accounts,
            data: RecordAction {
                args: RecordActionArgs { action: 42 },
            }.data(),
        };

        Ok(([instruction], vec![instruction_player]))
    }

    pub fn with_default(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let instruction_player = Keypair::new();
        let state_player = instruction_player.pubkey();

        Self::builder(
            svm,
            state_player,
            instruction_player,
            GameState::Active,
            GameSessionStatus::Active,
        )
    }

    pub fn with_invalid_player(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let state_player = Pubkey::new_unique();
        let instruction_player = Keypair::new(); // Unrecognized player

        Self::builder(
            svm,
            state_player,
            instruction_player,
            GameState::Active,
            GameSessionStatus::Active,
        )
    }

    pub fn with_inactive_game_session(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let instruction_player = Keypair::new();
        let state_player = instruction_player.pubkey();

        Self::builder(
            svm,
            state_player,
            instruction_player,
            GameState::Active,
            GameSessionStatus::Lost, // not active
        )
    }

    pub fn with_inactive_global_state(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let instruction_player = Keypair::new();
        let state_player = instruction_player.pubkey();

        Self::builder(
            svm,
            state_player,
            instruction_player,
            GameState::Locked, // not active
            GameSessionStatus::Active,
        )
    }
}


#[test]
fn test_record_action_success() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_default(&mut svm);

    let (instructions, signers) = match result {
        Ok(result) => result,
        Err(error) => panic!("Failed to create instruction: {}", error),
    };

    let payer = signers[0].pubkey();

    let recent_blockhash = svm.latest_blockhash();

    let transaction = Transaction::new_signed_with_payer(
        &instructions, Some(&payer), &signers, recent_blockhash
    );

    let result = svm.send_transaction(transaction);

    match result {
        Ok(result) => {
            println!("Program succeeded (compute units: {:?})", result.compute_units_consumed);
        }
        Err(error) => {
            println!("Program failed: {:?}", error);
            panic!("Expected success but transaction failed");
        }
    }
}

#[test]
fn test_record_action_fails_with_invalid_player() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_invalid_player(&mut svm);

    let (instructions, signers) = match result {
        Ok(result) => result,
        Err(error) => panic!("Failed to create instruction: {}", error),
    };

    let payer = signers[0].pubkey();

    let recent_blockhash = svm.latest_blockhash();

    let transaction = Transaction::new_signed_with_payer(&instructions, Some(&payer), &signers, recent_blockhash);

    let result = svm.send_transaction(transaction);

    match result {
        Ok(result) => {
            println!("Program succeeded (compute units: {:?})", result.compute_units_consumed);
            panic!("This transaction should have failed - Invalid player");
        }
        Err(error) => {
            println!("Program failed: {:?}", error);
            println!("Transaction failed successfully");
        }
    }
}

#[test]
fn test_record_action_fails_with_inactive_game_session() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_inactive_game_session(&mut svm);

    let (instructions, signers) = match result {
        Ok(result) => result,
        Err(error) => panic!("Failed to create instruction: {}", error),
    };

    let payer = signers[0].pubkey();

    let recent_blockhash = svm.latest_blockhash();

    let transaction = Transaction::new_signed_with_payer(&instructions, Some(&payer), &signers, recent_blockhash);

    let result = svm.send_transaction(transaction);

    match result {
        Ok(result) => {
            println!("Program succeeded (compute units: {:?})", result.compute_units_consumed);
            panic!("This transaction should have failed - Inactive game session");
        }
        Err(error) => {
            println!("Program failed: {:?}", error);
            println!("Transaction failed successfully");
        }
    }
}

#[test]
fn test_record_action_fails_with_inactive_global_state() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_inactive_global_state(&mut svm);

    let (instructions, signers) = match result {
        Ok(result) => result,
        Err(error) => panic!("Failed to create instruction: {}", error),
    };

    let payer = signers[0].pubkey();

    let recent_blockhash = svm.latest_blockhash();

    let transaction = Transaction::new_signed_with_payer(&instructions, Some(&payer), &signers, recent_blockhash);

    let result = svm.send_transaction(transaction);

    match result {
        Ok(result) => {
            println!("Program succeeded (compute units: {:?})", result.compute_units_consumed);
            panic!("This transaction should have failed - Inactive global state");
        }
        Err(error) => {
            println!("Program failed: {:?}", error);
            println!("Transaction failed successfully");
        }
    }
}
