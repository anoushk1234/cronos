use anchor_lang::{
    solana_program::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    },
    InstructionData,
};

pub fn admin_task_cancel(admin: Pubkey, config: Pubkey, task: Pubkey) -> Instruction {
    Instruction {
        program_id: cronos_scheduler::ID,
        accounts: vec![
            AccountMeta::new(admin, true),
            AccountMeta::new_readonly(config, false),
            AccountMeta::new(task, false),
        ],
        data: cronos_scheduler::instruction::AdminTaskCancel {}.data(),
    }
}
