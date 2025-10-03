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
    instruction::Withdraw,
    WithdrawArgs,
    GlobalState,
    GameState,
    ID as ZERO_FUN_PROGRAM_ID,
};


struct TestSetup {}

impl TestSetup {
    const ZERO_FUN_PROGRAM_ID: Pubkey = ZERO_FUN_PROGRAM_ID;

    pub fn with_default(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let instruction_admin = Keypair::new();

        Self::builder(svm, instruction_admin.pubkey(), instruction_admin)
    }

    pub fn with_invalid_admin(svm: &mut LiteSVM) -> Result<([Instruction; 1], Vec<Keypair>)> {
        let instruction_admin = Keypair::new();

        Self::builder(svm, Pubkey::new_unique(), instruction_admin)
    }

    fn builder(
        svm: &mut LiteSVM,
        state_admin: Pubkey,
        instruction_admin: Keypair,
    ) -> Result<([Instruction; 1], Vec<Keypair>)> {

        // Create the admin account
        svm.airdrop(&instruction_admin.pubkey(), 1_000_000_000).unwrap();

        // Create the PDAs
        let (global_state_key, _) =
            Pubkey::find_program_address(&[b"global-state"], &Self::ZERO_FUN_PROGRAM_ID);

        let (vault_key, vault_bump) =
            Pubkey::find_program_address(&[b"vault"], &Self::ZERO_FUN_PROGRAM_ID);

        // Create the global state account
        let global_state = GlobalState {
            admin: state_admin,
            message_signer: Pubkey::default(),
            max_deposit: 10u8,
            max_payout: 100u8,
            game_state: GameState::Active,
            vault_bump: vault_bump as u8,
        };

        create_global_state_account(svm, global_state_key, global_state);

        // Create vault account
        let rent = svm.minimum_balance_for_rent_exemption(0);
        let withdraw_amount: u64 = 500_000u64;
        create_vault_account(svm, vault_key, rent + withdraw_amount);

        let recipient_key = Pubkey::new_unique();

        // Build the instruction
        let accounts: Vec<AccountMeta> = vec![
            AccountMeta::new(global_state_key, false),
            AccountMeta::new(vault_key, false),
            AccountMeta::new(recipient_key, false),
            AccountMeta::new_readonly(instruction_admin.pubkey(), true),
            AccountMeta::new_readonly(SYSTEM_PROGRAM_ID, false),
        ];

        let args = WithdrawArgs { amount: withdraw_amount };

        let withdraw = Instruction {
            program_id: Self::ZERO_FUN_PROGRAM_ID,
            accounts,
            data: Withdraw { args }.data(),
        };

        Ok(([withdraw], vec![instruction_admin]))
    }
}

#[test]
fn test_withdraw_success() {
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
fn test_withdraw_fails_with_invalid_admin() {
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

    let result = svm.send_transaction(transaction);

    match result {
        Ok(result) => {
            println!("Program succeeded (compute units: {:?})", result.compute_units_consumed);
            panic!("This transaction should have failed - Invalid admin");
        }
        Err(error) => {
            println!("Program failed: {:?}", error);
            println!("Transaction failed successfully");
        }
    }
}
