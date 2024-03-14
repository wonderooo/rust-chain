## Toy implementation of blockchain based on Bitcoin in Rust
### What is implemented so far:
1. Mining of genesis block
2. Proof of work based on finding hash that fulfills requirements < 1 << (256b - 24b)
3. Transactions between addresses (strings as of now)
4. Persistance of transactions in the file byte based key value store
5. Generation of real bitcoin addresses
6. Cli
### Usage
#### Install
1. `cargo build --release`
2. `mv target/release/rust-chain .`
#### Commands
1. `./rust-chain --create-blockchain <address to transfer coins from mining genesis block>` - creates blockchain and saves db to file
2. `./rust-chain --print` - prints to stdout all transactions made in blockchain
3. `./rust-chain --send <address from> <address to> <value>` - sends coins from address to another
4. `./rust-chain --balance <address>` - check balance on given address
5. `./rust-chain --remove-blocks` - removes whole blockchain
6. `./rust-chain --address` - generates real unique bitcoin address