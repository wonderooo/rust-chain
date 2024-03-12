use std::fmt::Display;

use clap::Parser;
use rust_chain::{blockchain::Blockchain, Blockchainable};
use serde::{Deserialize, Serialize};

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
    /// Add block to blockchain with provided content
    #[arg(short, long)]
    add_block: Option<String>,

    /// Print all blocks in blockchain
    #[arg(short, long)]
    print: bool,

    /// Remove all blocks in blockchain
    #[arg(short, long)]
    remove_blocks: bool,
}

fn main() {
    let args = Args::parse();
    if let Some(content) = args.group.add_block {
        let mut blockchain = Blockchain::<Data>::new();
        blockchain.add_block(Data { field: content });
    }

    if args.group.print {
        let blockchain = Blockchain::<Data>::new();
        for block in blockchain {
            println!("{}", block);
        }
    }

    if args.group.remove_blocks {
        let blockchain = Blockchain::<Data>::new();
        blockchain.remove_blocks();
    }
}
