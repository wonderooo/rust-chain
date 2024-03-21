use std::fmt::Display;

use clap::Parser;
use rust_chain::{
    blockchain::Blockchain,
    wallet::{Wallet, Wallets},
    Blockchainable,
};
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

#[derive(Serialize, Deserialize, Debug)]
struct Data {
    field: String,
}

impl Blockchainable for Data {
    fn genesis_data() -> Self {
        Data {
            field: "GENESIS DATA".to_string(),
        }
    }
}

impl Display for Data {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.field)?;
        Ok(())
    }
}

/// Simple blockchain in rust
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[clap(flatten)]
    group: ArgGroup,
}

#[derive(Debug, clap::Args)]
#[group(required = true, multiple = false)]
struct ArgGroup {
    /// Create new blockchain with given address
    #[arg(short, long)]
    create_blockchain: Option<String>,

    /// Print all blocks in blockchain
    #[arg(short, long)]
    print: bool,

    /// Remove all blocks in blockchain
    #[arg(short, long)]
    remove_blocks: bool,

    /// Get balance at specified address
    #[arg(short, long)]
    balance: Option<String>,

    /// Send coins from an account to another (from, to, value)
    #[arg(short, long, num_args = 3)]
    send: Option<Vec<String>>,

    /// Get an valid bitcoin address
    #[arg(short, long)]
    address: bool,

    /// Create a wallet and save to file
    #[arg(short, long)]
    create_wallet: bool,

    /// Prints all wallets fetched from file
    #[arg(short, long)]
    print_wallets: bool,
}

fn main() {
    let args = Args::parse();

    if args.group.print {
        // TODO: Make address optional
        let blockchain = Blockchain::<Data>::new(&"".to_string());
        for block in blockchain {
            println!("{}", block);
        }
    }

    if args.group.remove_blocks {
        let blockchain = Blockchain::<Data>::new(&"".to_string());
        blockchain.remove_blocks();
    }

    if let Some(addr) = args.group.create_blockchain {
        Blockchain::<Data>::new(&addr);
    }

    if let Some(addr) = args.group.balance {
        let mut blockchain = Blockchain::<Data>::new(&"".to_string());
        let balance = blockchain.balance_at(&ByteBuf::from(addr.clone()));
        println!("Balance at {}: {}", addr, balance);
    }

    if let Some(v) = args.group.send {
        let mut blockchain = Blockchain::<Data>::new(&"".to_string());
        blockchain.send(
            &v[0],
            &v[1],
            v[2].parse::<u64>()
                .expect("Provided value is not a number!"),
        );
    }

    if args.group.address {
        let address = Wallet::new().address();
        println!(
            "{}",
            std::str::from_utf8(&address).expect("Could not create string address!")
        );
    }

    if args.group.create_wallet {
        let wallet = Wallet::new();
        Wallets::save_wallet(&wallet);
    }

    if args.group.print_wallets {
        let wallets = Wallets::fetch_wallets();
        println!("{}", wallets)
    }
}
