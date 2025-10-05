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
};

use zero_fun::{
    instruction::MarkGameAsWon,
    GameSession,
    GameSessionStatus,
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
        status: GameSessionStatus,
    ) -> Result<([Instruction; 1], Vec<Keypair>)> {

        // Create the player account
        svm.airdrop(&instruction_player.pubkey(), 1_000_000_000)
            .expect("Could not airdrop to player");

        // Set the game session state
        let (game_session, _) = Pubkey::find_program_address(
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
            status,
            public_config_seed: [0u8; HASH_LENGTH],
            game_metadata: "metadata".to_string(),
            player_moves: [0u8; zero_fun::MAX_MOVE_COUNT],
            vault: Pubkey::new_unique(),
            next_player_move_position: 0u8,
        };

        create_game_session_account(svm, game_session, &game_session_account);

        // Build the instruction
        let accounts: Vec<AccountMeta> = vec![
            AccountMeta::new(instruction_player.pubkey(), true),
            AccountMeta::new(game_session, false), 
        ];

        let instruction = Instruction {
            program_id: Self::ZERO_FUN_PROGRAM_ID,
            accounts,
            data: MarkGameAsWon {}.data(),
        };

        Ok(([instruction], vec![instruction_player]))
    }

    pub fn with_default(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let instruction_player = Keypair::new();
        let state_player = instruction_player.pubkey();

        // Status must be Active for success
        let status = GameSessionStatus::Active;

        Self::builder(svm, state_player, instruction_player, status)
    }

    pub fn with_invalid_player(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let state_player = Pubkey::new_unique();
        let instruction_player = Keypair::new(); // Not the owner of the game session

        let status = GameSessionStatus::Active;

        Self::builder(svm, state_player, instruction_player, status)
    }

    pub fn with_inactive_game(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let instruction_player = Keypair::new();
        let state_player = instruction_player.pubkey();

        // Game is not active, should fail
        let status = GameSessionStatus::Lost;

        Self::builder(svm, state_player, instruction_player, status)
    }
}


#[test]
fn test_mark_game_as_won_success() {
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
fn test_mark_game_as_won_fails_with_invalid_player() {
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
fn test_mark_game_as_won_fails_with_inactive_game() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_inactive_game(&mut svm);

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
            panic!("This transaction should have failed - Game not active");
        }
        Err(error) => {
            println!("Program failed: {:?}", error);
            println!("Transaction failed successfully");
        }
    }
}
