pub mod errors;

use {
    errors::TestError,
    serde::{Deserialize as JsonDeserialize, Serialize as JsonSerialize},
    solana_sdk::{
        instruction::{AccountMeta, Instruction},
        pubkey::Pubkey,
    },
    std::str::FromStr,
};

#[derive(Debug, JsonDeserialize, JsonSerialize)]
pub struct JsonInstructionData {
    pub program_id: String,
    pub accounts: Vec<JsonAccountMetaData>,
    pub data: Vec<u8>,
}

impl TryFrom<&JsonInstructionData> for Instruction {
    type Error = TestError;

    fn try_from(value: &JsonInstructionData) -> Result<Self, Self::Error> {
        Ok(Instruction {
            program_id: Pubkey::from_str(value.program_id.as_str())
                .map_err(|_err| TestError::BadParameter("asdf".into()))?,
            accounts: value
                .accounts
                .iter()
                .map(|ix| AccountMeta::try_from(ix).unwrap())
                .collect::<Vec<AccountMeta>>(),
            data: value.data.clone(),
        })
    }
}

#[derive(Debug, JsonDeserialize, JsonSerialize, PartialEq)]
pub struct JsonAccountMetaData {
    pub pubkey: String,
    pub is_signer: bool,
    pub is_writable: bool,
}

impl TryFrom<&JsonAccountMetaData> for AccountMeta {
    type Error = TestError;

    fn try_from(value: &JsonAccountMetaData) -> Result<Self, Self::Error> {
        Ok(AccountMeta {
            pubkey: Pubkey::from_str(value.pubkey.as_str())
                .map_err(|_err| TestError::BadParameter("asdf".into()))?,
            is_signer: value.is_signer,
            is_writable: value.is_writable,
        })
    }
}

#[cfg(test)]
mod tests {
    use {
        crate::{errors::TestError, JsonInstructionData},
        serde_json::json,
        solana_client_helpers::{Client, RpcClient},
        solana_sdk::{
            commitment_config::CommitmentConfig, instruction::Instruction,
            native_token::LAMPORTS_PER_SOL, signature::Keypair, transaction::Transaction,
        },
        std::sync::Arc,
    };

    fn new_client() -> Arc<Client> {
        let url = "http://localhost:8899";
        let client = Arc::new(Client {
            client: RpcClient::new_with_commitment(url, CommitmentConfig::confirmed()),
            payer: Keypair::new(),
        });
        client
            .airdrop(&client.payer_pubkey(), LAMPORTS_PER_SOL)
            .unwrap();
        client
    }

    fn sign_and_submit(client: &Arc<Client>, ixs: &[Instruction]) {
        let mut tx = Transaction::new_with_payer(ixs, Some(&client.payer_pubkey()));
        tx.sign(&vec![&client.payer], client.latest_blockhash().unwrap());
        let sig = client.send_and_confirm_transaction(&tx).unwrap();
        println!("Signature: {}", sig);
    }

    #[test]
    #[ignore]
    fn initialize() {
        let client = new_client();
        let authority_pda = cronos_sdk::scheduler::state::Authority::pda();
        let config_pda = cronos_sdk::scheduler::state::Config::pda();
        let daemon_pda = cronos_sdk::scheduler::state::Daemon::pda(authority_pda.0);
        let fee_pda = cronos_sdk::scheduler::state::Fee::pda(daemon_pda.0);
        let ix = cronos_sdk::scheduler::instruction::admin_initialize(
            client.payer_pubkey(),
            authority_pda,
            config_pda,
            daemon_pda,
            fee_pda,
        );
        sign_and_submit(&client, &[ix]);
        assert_eq!(2 + 2, 4);
    }

    #[test]
    #[ignore]
    fn plugin_bench_single_task() {
        let client = new_client();
        let owner = client.payer_pubkey();

        let daemon_pda = cronos_sdk::scheduler::state::Daemon::pda(owner);
        let fee_pda = cronos_sdk::scheduler::state::Fee::pda(daemon_pda.0);

        let ix = cronos_sdk::scheduler::instruction::daemon_new(daemon_pda, fee_pda, owner);
        sign_and_submit(&client, &[ix]);

        let data = client
            .get_account_data(&daemon_pda.0)
            .map_err(|_err| TestError::AccountNotFound(daemon_pda.0.to_string()))
            .unwrap();

        let daemon_data = cronos_sdk::scheduler::state::Daemon::try_from(data)
            .map_err(|_err| TestError::AccountDataNotParsable(daemon_pda.0.to_string()))
            .unwrap();

        assert_eq!(daemon_data.owner, owner);
        assert_eq!(daemon_data.task_count, 0);

        let daemon_addr = daemon_pda.0;

        let memo = json!({
          "program_id": "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
          "accounts": [
            {
              "pubkey": daemon_addr.to_string(),
              "is_signer": true,
              "is_writable": false
            }
          ],
          "data": [72, 101, 108, 108, 111, 44, 32, 119, 111, 114, 108, 100, 33]
        });

        let ix_json = serde_json::from_value::<JsonInstructionData>(memo)
            .expect("JSON was not well-formatted");

        let ix = Instruction::try_from(&ix_json).unwrap();

        let task_pda = cronos_sdk::scheduler::state::Task::pda(daemon_addr, daemon_data.task_count);

        let task_ix = cronos_sdk::scheduler::instruction::task_new(
            task_pda,
            daemon_pda.0,
            owner,
            vec![ix],
            "* * * * * * *".to_string(),
        );

        sign_and_submit(&client, &[task_ix]);

        let data = client
            .get_account_data(&task_pda.0)
            .map_err(|_err| TestError::AccountDataNotParsable(task_pda.0.to_string()))
            .unwrap();

        let task_data = cronos_sdk::scheduler::state::Task::try_from(data)
            .map_err(|_err| TestError::AccountNotFound(task_pda.0.to_string()))
            .unwrap();

        let data = client
            .get_account_data(&daemon_pda.0)
            .map_err(|_err| TestError::AccountNotFound(daemon_pda.0.to_string()))
            .unwrap();

        let daemon_data = cronos_sdk::scheduler::state::Daemon::try_from(data)
            .map_err(|_err| TestError::AccountDataNotParsable(daemon_pda.0.to_string()))
            .unwrap();

        assert_eq!(daemon_data.task_count, 1);
        assert_eq!(task_data.daemon, daemon_addr);
    }

    #[test]
    #[ignore]
    fn plugin_bench_1_daemon_100_tasks() {
        let client = new_client();
        let owner = client.payer_pubkey();

        let daemon_pda = cronos_sdk::scheduler::state::Daemon::pda(owner);
        let daemon_addr = daemon_pda.0;
        let fee_pda = cronos_sdk::scheduler::state::Fee::pda(daemon_addr);

        let ix = cronos_sdk::scheduler::instruction::daemon_new(daemon_pda, fee_pda, owner);

        sign_and_submit(&client, &[ix]);

        let data = client
            .get_account_data(&daemon_addr)
            .map_err(|_err| TestError::AccountNotFound(daemon_addr.to_string()))
            .unwrap();

        let daemon_data = cronos_sdk::scheduler::state::Daemon::try_from(data)
            .map_err(|_err| TestError::AccountDataNotParsable(daemon_addr.to_string()))
            .unwrap();

        assert_eq!(daemon_data.task_count, 0);

        let memo = json!({
          "program_id": "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
          "accounts": [
            {
              "pubkey": daemon_addr.to_string(),
              "is_signer": true,
              "is_writable": false
            }
          ],
          "data": [72, 101, 108, 108, 111, 44, 32, 119, 111, 114, 108, 100, 33]
        });

        let ix_json = serde_json::from_value::<JsonInstructionData>(memo)
            .expect("JSON was not well-formatted");

        for _i in 0..100 {
            let ix = Instruction::try_from(&ix_json).unwrap();

            let data = client
                .get_account_data(&daemon_addr)
                .map_err(|_err| TestError::AccountNotFound(daemon_addr.to_string()))
                .unwrap();

            let daemon_data = cronos_sdk::scheduler::state::Daemon::try_from(data)
                .map_err(|_err| TestError::AccountDataNotParsable(daemon_addr.to_string()))
                .unwrap();

            let task_pda =
                cronos_sdk::scheduler::state::Task::pda(daemon_addr, daemon_data.task_count);

            let task_ix = cronos_sdk::scheduler::instruction::task_new(
                task_pda,
                daemon_addr,
                owner,
                vec![ix],
                "* * * * * *".to_string(),
            );

            sign_and_submit(&client, &[task_ix]);
        }

        let data = client
            .get_account_data(&daemon_addr)
            .map_err(|_err| TestError::AccountNotFound(daemon_addr.to_string()))
            .unwrap();

        let daemon_data = cronos_sdk::scheduler::state::Daemon::try_from(data)
            .map_err(|_err| TestError::AccountDataNotParsable(daemon_addr.to_string()))
            .unwrap();

        assert_eq!(daemon_data.task_count, 100);
    }

    #[test]
    #[ignore]
    fn plugin_bench_10_daemons_10_tasks() {
        for _i in 0..10 {
            let client = new_client();
            let owner = client.payer_pubkey();

            let daemon_pda = cronos_sdk::scheduler::state::Daemon::pda(owner);
            let daemon_addr = daemon_pda.0;
            let fee_pda = cronos_sdk::scheduler::state::Fee::pda(daemon_addr);

            let ix = cronos_sdk::scheduler::instruction::daemon_new(daemon_pda, fee_pda, owner);

            sign_and_submit(&client, &[ix]);

            let data = client
                .get_account_data(&daemon_addr)
                .map_err(|_err| TestError::AccountNotFound(daemon_addr.to_string()))
                .unwrap();

            let daemon_data = cronos_sdk::scheduler::state::Daemon::try_from(data)
                .map_err(|_err| TestError::AccountDataNotParsable(daemon_addr.to_string()))
                .unwrap();

            assert_eq!(daemon_data.task_count, 0);

            let memo = json!({
              "program_id": "MemoSq4gqABAXKb96qnH8TysNcWxMyWCqXgDLGmfcHr",
              "accounts": [
                {
                  "pubkey": daemon_addr.to_string(),
                  "is_signer": true,
                  "is_writable": false
                }
              ],
              "data": [72, 101, 108, 108, 111, 44, 32, 119, 111, 114, 108, 100, 33]
            });

            let ix_json = serde_json::from_value::<JsonInstructionData>(memo)
                .expect("JSON was not well-formatted");

            for _i in 0..10 {
                let ix = Instruction::try_from(&ix_json).unwrap();

                let data = client
                    .get_account_data(&daemon_addr)
                    .map_err(|_err| TestError::AccountNotFound(daemon_addr.to_string()))
                    .unwrap();

                let daemon_data = cronos_sdk::scheduler::state::Daemon::try_from(data)
                    .map_err(|_err| TestError::AccountDataNotParsable(daemon_addr.to_string()))
                    .unwrap();

                let task_pda =
                    cronos_sdk::scheduler::state::Task::pda(daemon_addr, daemon_data.task_count);

                let task_ix = cronos_sdk::scheduler::instruction::task_new(
                    task_pda,
                    daemon_addr,
                    owner,
                    vec![ix],
                    "* * * * * *".to_string(),
                );

                sign_and_submit(&client, &[task_ix]);
            }

            let data = client
                .get_account_data(&daemon_addr)
                .map_err(|_err| TestError::AccountNotFound(daemon_addr.to_string()))
                .unwrap();

            let daemon_data = cronos_sdk::scheduler::state::Daemon::try_from(data)
                .map_err(|_err| TestError::AccountDataNotParsable(daemon_addr.to_string()))
                .unwrap();

            assert_eq!(daemon_data.task_count, 10);
        }
    }
}
