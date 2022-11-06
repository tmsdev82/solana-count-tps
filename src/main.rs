use dotenv::dotenv;
use solana_client::{rpc_client::RpcClient, rpc_config::RpcBlockConfig};
use solana_transaction_status::{EncodedConfirmedBlock, UiTransactionEncoding};

fn count_user_transactions(block: &EncodedConfirmedBlock) -> u64 {
    let mut user_transactions_count: u64 = 0;

    for transaction_status in &block.transactions {
        let transaction = transaction_status.transaction.decode().unwrap();
        let account_keys = transaction.message.static_account_keys();

        let mut num_vote_instructions = 0;
        for instruction in transaction.message.instructions() {
            let program_id_index = instruction.program_id_index;
            let program_id = account_keys[usize::from(program_id_index)];

            if program_id == solana_sdk::vote::program::id() {
                num_vote_instructions += 1;
                log::debug!("Found vote instruction");
            } else {
                log::debug!("Non-vote instruction");
            }
        }
        if num_vote_instructions == transaction.message.instructions().len() {
            log::debug!("It's a vote transaction");
        } else {
            log::debug!("It's a user transaction");
            user_transactions_count += 1;
        }
    }

    let vote_transactions_count = block
        .transactions
        .len()
        .checked_sub(user_transactions_count as usize)
        .expect("underflow");

    log::debug!("solana total txs: {}", block.transactions.len());
    log::debug!("solana user txs: {}", user_transactions_count);
    log::debug!("solana vote txs: {}", vote_transactions_count);

    user_transactions_count
}

fn get_block(client: &RpcClient, block_num: u64) -> EncodedConfirmedBlock {
    log::debug!("Getting block number: {}", block_num);

    let config = RpcBlockConfig {
        encoding: Some(UiTransactionEncoding::Base64),
        max_supported_transaction_version: Some(0),
        ..Default::default()
    };

    let block = client.get_block_with_config(block_num, config);
    let encoded_block: EncodedConfirmedBlock = block.unwrap().into();

    encoded_block
}

fn main() {
    dotenv().ok();
    env_logger::init();

    log::info!("Solana count transactions per second!");

    let client = RpcClient::new("https://solana-api.projectserum.com");

    let solana_version = client.get_version().unwrap().solana_core;
    log::info!("Solana version: {}", &solana_version);

    let latest_block_number = client.get_slot().unwrap();
    let block = get_block(&client, latest_block_number);
    let user_transactions_count = count_user_transactions(&block);
}
