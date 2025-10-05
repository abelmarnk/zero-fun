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
    create_vault_account,
    set_current_time
};

use zero_fun::{
    instruction::DefaultGame,
    GameSession,
    GameSessionStatus,
    DEFAULT_OFFSET,
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
        state_vault: Pubkey,
        instruction_vault: Pubkey,
        last_action_time: i64,
        current_time: i64,
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
            last_action_time,
            player: state_player,
            deposit: 1_000_000u64,
            status: GameSessionStatus::Active,
            public_config_seed:[0u8; HASH_LENGTH],
            game_metadata: "metadata".to_string(),
            player_moves: [0u8; zero_fun::MAX_MOVE_COUNT],
            vault: state_vault,
            next_player_move_position: 0u8,
        };

        create_game_session_account(svm, game_session, &game_session_account);

        // Set the vault state
        let deposit_amount = 1_000_000u64;

        let rent = svm.minimum_balance_for_rent_exemption(0);

        create_vault_account(svm, instruction_vault, rent + deposit_amount);

        // Set the value for Clock::get()?.unix_timestamp
        set_current_time(svm, current_time);

        // Build the instruction
        let accounts: Vec<AccountMeta> = vec![
            AccountMeta::new(instruction_player.pubkey(), true),
            AccountMeta::new(instruction_vault, false),   
            AccountMeta::new(game_session, false), 
        ];

        let instruction = Instruction {
            program_id: Self::ZERO_FUN_PROGRAM_ID,
            accounts,
            data: DefaultGame {}.data(),
        };

        Ok(([instruction], vec![instruction_player]))
    }

    pub fn with_default(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let instruction_player = Keypair::new();
        let state_player = instruction_player.pubkey();
        let vault = Pubkey::new_unique();

        // Current time
        let current_time = 123456789i64;

        // Last action time should be < (current_time - DEFAULT_OFFSET) for success
        let last_action_time = (current_time - DEFAULT_OFFSET) - 10;

        Self::builder(
            svm,
            state_player,
            instruction_player,
            vault,
            vault,
            last_action_time,
            current_time,
        )
    }

    pub fn with_invalid_vault(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let instruction_player = Keypair::new();
        let state_player = instruction_player.pubkey();

        let state_vault = Pubkey::new_unique();
        let instruction_vault = Pubkey::new_unique(); // Unrecognized vault

        let current_time = 123456789i64;

        let last_action_time = (current_time - DEFAULT_OFFSET) - 10;

        Self::builder(
            svm,
            state_player,
            instruction_player,
            state_vault,
            instruction_vault,
            last_action_time,
            current_time,
        )
    }

    pub fn with_invalid_player(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let state_player = Pubkey::new_unique();
        let instruction_player = Keypair::new(); // Unrecognized player

        let vault = Pubkey::new_unique();

        let current_time = 123456789i64;

        let last_action_time = (current_time - DEFAULT_OFFSET) - 10;

        Self::builder(
            svm,
            state_player,
            instruction_player,
            vault,
            vault,
            last_action_time,
            current_time,
        )
    }

    pub fn with_too_soon_to_default(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let instruction_player = Keypair::new();
        let state_player = instruction_player.pubkey();

        let vault = Pubkey::new_unique();

        let current_time = 123456789i64;

        // Here the last action time is greater than the distance from the offset 
        // so it should fail
        let last_action_time = (current_time - DEFAULT_OFFSET) + 10;

        Self::builder(
            svm,
            state_player,
            instruction_player,
            vault,
            vault,
            last_action_time,
            current_time,
        )
    }
}


#[test]
fn test_default_game_success() {
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
fn test_default_game_fails_with_invalid_vault() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_invalid_vault(&mut svm);

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
            panic!("This transaction should have failed - Invalid vault");
        }
        Err(error) => {
            println!("Program failed: {:?}", error);
            println!("Transaction failed successfully");
        }
    }
}

#[test]
fn test_default_game_fails_with_invalid_player() {
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
fn test_default_game_fails_when_too_soon_to_default() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_too_soon_to_default(&mut svm);

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
            panic!("This transaction should have failed - Too soon to default");
        }
        Err(error) => {
            println!("Program failed: {:?}", error);
            println!("Transaction failed successfully");
        }
    }
}
