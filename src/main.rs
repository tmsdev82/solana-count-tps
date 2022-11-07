use chrono::{DateTime, NaiveDateTime, Utc};
use dotenv::dotenv;
use solana_client::{rpc_client::RpcClient, rpc_config::RpcBlockConfig};
use solana_transaction_status::{EncodedConfirmedBlock, UiTransactionEncoding};

fn calculate_for_range(client: &RpcClient, threshold_seconds: i64) {
    let calculation_start = Utc::now();

    let newest_block_number = client.get_slot().unwrap();
    let mut current_block = get_block(client, newest_block_number);

    let newest_timestamp = current_block.block_time.unwrap();
    let timestamp_threshold = newest_timestamp.checked_sub(threshold_seconds).unwrap();

    let mut total_transactions_count: u64 = 0;

    let oldest_timestamp = loop {
        let prev_block_number = current_block.parent_slot;
        let prev_block = get_block(client, prev_block_number);

        let transactions_count = count_user_transactions(&current_block);
        let naive_datetime = NaiveDateTime::from_timestamp(current_block.block_time.unwrap(), 0);
        let utc_dt: DateTime<Utc> = DateTime::from_utc(naive_datetime, Utc);

        log::debug!("Block time: {}", utc_dt.format("%Y-%m-%d %H:%M:%S"));

        total_transactions_count = total_transactions_count
            .checked_add(transactions_count)
            .expect("overflow");

        let prev_block_timestamp = prev_block.block_time.unwrap();

        if prev_block_timestamp <= timestamp_threshold {
            break prev_block_timestamp;
        }

        if prev_block.block_height.unwrap() == 0 {
            break prev_block_timestamp;
        }

        current_block = prev_block;
    };

    let transactions_per_second =
        calculate_tps(oldest_timestamp, newest_timestamp, total_transactions_count);
    let calculation_end = Utc::now();

    let duration = calculation_end
        .signed_duration_since(calculation_start)
        .to_std()
        .unwrap();

    log::info!("calculation took: {} seconds", duration.as_secs());
    log::info!(
        "total transactions per second over period: {}",
        transactions_per_second
    );
}

fn calculate_tps(oldest_timestamp: i64, newest_timestamp: i64, transactions_count: u64) -> f64 {
    let total_seconds_diff = newest_timestamp.saturating_sub(oldest_timestamp);

    let total_seconds_diff_f64 = total_seconds_diff as f64;
    let transaction_count_f64 = transactions_count as f64;

    let mut transactions_per_second = transaction_count_f64 / total_seconds_diff_f64;

    if transactions_per_second.is_nan() || transactions_per_second.is_infinite() {
        transactions_per_second = 0.0;
    }

    transactions_per_second
}

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

    calculate_for_range(&client, 60 * 1);
}
