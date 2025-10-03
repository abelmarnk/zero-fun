use anchor_lang::InstructionData;
use litesvm::LiteSVM;
use anyhow::Result;
use solana_sdk::{
    instruction::{Instruction, AccountMeta},
    pubkey::Pubkey,
    signer::{Signer, keypair::Keypair},
    transaction::Transaction,
    system_program::ID as SYSTEM_PROGRAM_ID,
};

mod common;
use common::utils::{
    add_zero_fun_program,
    create_global_state_account,
    create_vault_account,
};

use zero_fun::{
    GameState, GlobalState, HASH_LENGTH, ID as ZERO_FUN_PROGRAM_ID, InitializeGameArgs, MAX_METADATA_LENGTH, instruction::InitializeGame
};

// Here what is relevant is that the player should have signed(the system program would test this), 
// the metadata should be within bounds, the game is active as the deposit is within bounds.
// Other stuff is filled with defaults.

struct TestSetup {}

impl TestSetup {
    const ZERO_FUN_PROGRAM_ID: Pubkey = ZERO_FUN_PROGRAM_ID;

    const SYSTEM_PROGRAM_ID: Pubkey = SYSTEM_PROGRAM_ID;

    fn builder(
        svm: &mut LiteSVM,
        metadata: String,
        deposit: u64,
        vault_balance: u64,
        max_deposit_bps: u8,
        game_state: GameState,
    ) -> Result<([Instruction; 1], Vec<Keypair>)> {

        // Create the player account
        let player = Keypair::new();
        svm.airdrop(&player.pubkey(), 1_000_000_000).expect("Could not airdrop to player");

        // Create the PDAs
        let (game_session, _) = Pubkey::find_program_address(
            &[
                b"game-session",
                [0u8; HASH_LENGTH].as_ref(),
                player.pubkey().as_ref(),
            ],
            &Self::ZERO_FUN_PROGRAM_ID,
        );

        let (global_state, _) =
            Pubkey::find_program_address(&[b"global-state"], &Self::ZERO_FUN_PROGRAM_ID);

        let (vault, vault_bump) =
            Pubkey::find_program_address(&[b"vault"], &Self::ZERO_FUN_PROGRAM_ID);

        let (user_vault, _) = Pubkey::find_program_address(
            &[
                b"vault",
                [0u8; HASH_LENGTH].as_ref(),
                player.pubkey().as_ref(),
            ],
            &Self::ZERO_FUN_PROGRAM_ID,
        );        

        // Create the global state account
        let global_state_account = GlobalState {
            admin: Pubkey::default(),
            message_signer: Pubkey::default(),
            max_deposit: max_deposit_bps,
            max_payout: 100u8,
            game_state,
            vault_bump: vault_bump as u8,
        };

        create_global_state_account(svm, global_state, global_state_account);

        // Create the vault account
        create_vault_account(svm, vault, vault_balance);

        // Build the instruction
        let accounts: Vec<AccountMeta> = vec![
            AccountMeta::new(game_session, false),
            AccountMeta::new(player.pubkey(), true),
            AccountMeta::new(user_vault, false),
            AccountMeta::new_readonly(vault, false),
            AccountMeta::new_readonly(global_state, false),
            AccountMeta::new_readonly(Self::SYSTEM_PROGRAM_ID, false),
        ];

        let args = InitializeGameArgs {
            public_config_seed:[0u8; HASH_LENGTH],
            game_metadata: metadata,
            deposit,
        };

        let initialize_game = Instruction {
            program_id: Self::ZERO_FUN_PROGRAM_ID,
            accounts,
            data: InitializeGame { args }.data(),
        };

        Ok(([initialize_game], vec![player]))
    }

    pub fn with_default(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let metadata = "V0".to_string();
        let deposit = 10_000u64;
        let vault_balance = 1_000_000_000u64;
        let max_deposit_bps = 10u8;
        let game_state = GameState::Active;

        Self::builder(svm, metadata, deposit, vault_balance, max_deposit_bps, game_state)
    }

    pub fn with_metadata_too_long(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let metadata = "0".repeat(MAX_METADATA_LENGTH + 1); // Metadata exceeds max by 1
        let deposit = 10_000u64;
        let vault_balance = 1_000_000_000u64;
        let max_deposit_bps = 10u8;
        let game_state = GameState::Active;

        Self::builder(svm, metadata, deposit, vault_balance, max_deposit_bps, game_state)
    }

    pub fn with_deposit_exceeds_max(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let metadata = "V0".to_string();
        let deposit = 101u64; // will exceed computed max deposit
        let vault_balance = 1_000_000u64;
        let max_deposit_bps = 1u8;
        let game_state = GameState::Active;

        Self::builder(svm, metadata, deposit, vault_balance, max_deposit_bps, game_state)
    }

    pub fn with_game_not_active(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let metadata = "V0".to_string();
        let deposit = 10_000u64;
        let vault_balance = 1_000_000_000u64;
        let max_deposit_bps = 10u8;
        let game_state = GameState::Locked; // Games can only be created when the game is active

        Self::builder(svm, metadata, deposit, vault_balance, max_deposit_bps, game_state)
    }
}


#[test]
fn test_initialize_game_success() {
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
        &instructions, Some(&payer), &signers, recent_blockhash,
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
fn test_initialize_game_fails_when_metadata_too_long() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_metadata_too_long(&mut svm);

    let (instructions, signers) = match result {
        Ok(result) => result,

        Err(error) => panic!("Failed to create instruction: {}", error),
    };

    let payer = signers[0].pubkey();

    let recent_blockhash = svm.latest_blockhash();
    
    let transaction = Transaction::new_signed_with_payer(
        &instructions, Some(&payer), &signers, recent_blockhash,
    );

    let result = svm.send_transaction(transaction);

    match result {
        Ok(result) => {
            println!("Program succeeded (compute units: {:?})", result.compute_units_consumed);
            panic!("This transaction should have failed - Metadata too long");
        }
        Err(error) => {
            println!("Program failed: {:?}", error);
            println!("Transaction failed successfully");
        }
    }
}

#[test]
fn test_initialize_game_fails_when_deposit_exceeds_max() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_deposit_exceeds_max(&mut svm);

    let (instructions, signers) = match result {
        Ok(result) => result,

        Err(error) => panic!("Failed to create instruction: {}", error),
    };

    let payer = signers[0].pubkey();

    let recent_blockhash = svm.latest_blockhash();
    
    let transaction = Transaction::new_signed_with_payer(
        &instructions, Some(&payer), &signers, recent_blockhash,
    );

    let result = svm.send_transaction(transaction);

    match result {
        Ok(result) => {
            println!("Program succeeded (compute units: {:?})", result.compute_units_consumed);
            panic!("This transaction should have failed - Deposit exceeds maximum");
        }
        Err(error) => {
            println!("Program failed: {:?}", error);
            println!("Transaction failed successfully");
        }
    }
}

#[test]
fn test_initialize_game_fails_when_game_not_active() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_game_not_active(&mut svm);

    let (instructions, signers) = match result {
        Ok(result) => result,

        Err(error) => panic!("Failed to create instruction: {}", error),
    };

    let payer = signers[0].pubkey();

    let recent_blockhash = svm.latest_blockhash();
    
    let transaction = Transaction::new_signed_with_payer(
        &instructions, Some(&payer), &signers, recent_blockhash,
    );

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
