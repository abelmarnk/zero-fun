use anchor_lang::InstructionData;
use litesvm::LiteSVM;
use anyhow::Result;
use solana_sdk::{
    instruction::{Instruction, AccountMeta},
    pubkey::Pubkey,
    signer::{Signer, keypair::Keypair},
    transaction::Transaction,
};

mod common;

use common::utils::{
    assert_custom_transaction_error,
    assert_transaction_error,
    assert_transaction_success,
    add_zero_fun_program,
    create_global_state_account,
};

use zero_fun::{
    instruction::UpdateGlobalState,
    UpdateGlobalStateArgs,
    GlobalStateUpdate,
    GameState,
    GlobalState,
    ID as ZERO_FUN_PROGRAM_ID,
};

use solana_sdk::transaction::TransactionError;

use crate::common::disable_signer;

// Here what is important is that the global state can only be updated by the admin, 
// and their signature is required, other stuff is filled with defaults
struct TestSetup {}

impl TestSetup {
    const ZERO_FUN_PROGRAM_ID: Pubkey = ZERO_FUN_PROGRAM_ID;

    pub fn with_default(svm: &mut LiteSVM) 
        -> Result<([Instruction; 1], Vec<Keypair>)> {
        let admin = Keypair::new();

        Self::builder(svm, admin.insecure_clone(), admin)
    }

    pub fn with_invalid_admin(svm: &mut LiteSVM) 
        -> Result<([Instruction; 1], Vec<Keypair>)> {
        let admin = Keypair::new();

        let invalid_admin = Keypair::new();

        Self::builder(svm, admin, invalid_admin)
    }

    fn builder(svm: &mut LiteSVM, state_admin: Keypair, instruction_admin: Keypair) 
        -> Result<([Instruction; 1], Vec<Keypair>)> {

        // Create the admin account
        svm.airdrop(&state_admin.pubkey(), 1_000_000_000).unwrap();

        // Create the global state account
        let (global_state_key, _) =
            Pubkey::find_program_address(
                &[b"global-state"], &Self::ZERO_FUN_PROGRAM_ID
            );

        let global_state = GlobalState {
            admin: state_admin.pubkey(),
            message_signer: Pubkey::default(),
            max_deposit: 10u8,
            max_payout: 100u8,
            game_state: GameState::Active,
            vault_bump: 255u8,
        };

        create_global_state_account(svm, global_state_key, global_state);

        // Build the instruction
        let accounts = vec![
            AccountMeta::new(global_state_key, false),
            AccountMeta::new_readonly(instruction_admin.pubkey(), true),
        ];

        let args = UpdateGlobalStateArgs {
            update: GlobalStateUpdate::MaxDeposit(50),
        };

        let update_state = Instruction {
            program_id: Self::ZERO_FUN_PROGRAM_ID,
            accounts,
            data: UpdateGlobalState { args }.data(),
        };

        Ok(([update_state], vec![instruction_admin]))
    }
}

#[test]
fn test_update_global_state_success() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_default(&mut svm);

    let (instructions, signers) = match result {
        Ok(result) => result,

        Err(error) => panic!("Failed to create instruction: {}", error),
    };

    let recent_blockhash = svm.latest_blockhash();

    let payer = signers[0].pubkey();
    
    let transaction = Transaction::new_signed_with_payer(
        &instructions, Some(&payer), &signers, recent_blockhash,
    );

    assert_transaction_success(svm.send_transaction(transaction));
}

#[test]
fn test_update_global_state_fails_with_invalid_admin() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_invalid_admin(&mut svm);

    let (instructions, signers) = match result {
        Ok(result) => result,

        Err(error) => panic!("Failed to create instruction: {}", error),
    };

    let recent_blockhash = svm.latest_blockhash();

    let payer = signers[0].pubkey();
    
    let transaction = Transaction::new_signed_with_payer(
        &instructions, Some(&payer), &signers, recent_blockhash,
    );

    assert_custom_transaction_error(
        svm.send_transaction(transaction),
        zero_fun::GameError::InvalidAdmin,
    );
}

#[test]
fn test_update_global_state_fails_when_admin_does_not_sign() {
    let mut svm = LiteSVM::new();

    add_zero_fun_program(&mut svm);

    let result = TestSetup::with_default(&mut svm);

    let (mut instructions, mut signers) = match result {
        Ok(result) => result,

        Err(error) => panic!("Failed to create instruction: {}", error),
    };

    disable_signer(&mut instructions[0], signers[0].pubkey());
    
    let recent_blockhash = svm.latest_blockhash();
    
    let payer = Keypair::new();

    let payer_key = payer.pubkey();
    
    signers[0] = payer;
    
    let transaction = Transaction::new_signed_with_payer(
        &instructions, Some(&payer_key), &signers, recent_blockhash,
    );

    assert_transaction_error(
        svm.send_transaction(transaction),
        TransactionError::SignatureFailure,
    );
}
