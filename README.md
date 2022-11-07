# Solana count transactions per second

This project is an example of how to count the average number of transactions on the Solana blockchain. The program retrieves blocks up until a given time into the past and then calculates the average transactions per second. Vote transactions are not counted.

My blog post explaining the code can be found here: [Solana transactions per second: how to with Rust](https://tms-dev-blog.com/solana-transactions-per-second-with-rust/).

## Running the program

Simply run using cargo:

```
cargo run --release
```

No additional configuration is needed. The program is hard coded to calculate the transactions per second over the past minute. However, this can easily be adjusted by changing this line:

```rust
calculate_for_range(&client, 60 * 1);
```

The second parameter here is time in seconds backwards in time for the current time.
