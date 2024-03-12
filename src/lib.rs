use std::{
    fmt::Display,
    marker::PhantomData,
    time::{SystemTime, UNIX_EPOCH},
};

use bincode::{deserialize, serialize};
use num_bigint::BigUint;
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_bytes::ByteBuf;
use sha2::{Digest, Sha256};
use sled::Db;

pub trait Blockchainable: Serialize + DeserializeOwned + Display {
    fn genesis_data() -> Self;
}

#[derive(Serialize, Deserialize)]
pub struct Block<T> {
    pub timestamp: SystemTime,
    pub data: T,
    #[serde(with = "serde_bytes")]
    pub previous_block_hash: Option<ByteBuf>,
    #[serde(with = "serde_bytes")]
    pub hash: Option<ByteBuf>,
    pub nonce: Option<u64>,
}

impl<T> Block<T> {
    pub fn new(data: T, previous_block_hash: Option<ByteBuf>) -> Self
    where
        T: Blockchainable,
    {
        let mut block = Block {
            timestamp: SystemTime::now(),
            data,
            previous_block_hash: previous_block_hash,
            hash: None,
            nonce: None,
        };

        let pow = ProofOfWork::new(&block);
        if let Some((nonce, hash)) = pow.run() {
            block.hash = Some(hash);
            block.nonce = Some(nonce);
        }

        block
    }
}

impl<T> Display for Block<T>
where
    T: Blockchainable,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Block")?;
        writeln!(f, "\tDATA: {}", self.data)?;
        writeln!(
            f,
            "\tTIMESTAMP (NANOS): {}",
            self.timestamp
                .duration_since(UNIX_EPOCH)
                .expect("Could not calculate elapsed time!")
                .as_nanos()
        )?;
        if let Some(prev) = self.previous_block_hash.clone() {
            writeln!(f, "\tPREVIOUS HASH: {}", hex::encode(prev))?
        } else {
            writeln!(f, "\tPREVIOUS HASH: NOTHING YET")?
        }
        if let Some(hash) = self.hash.clone() {
            writeln!(f, "\tHASH: {}", hex::encode(hash))?
        } else {
            writeln!(f, "\tHASH: NOTHING YET")?
        }
        if let Some(nonce) = self.nonce {
            writeln!(f, "\tNONCE: {}", nonce)?
        } else {
            writeln!(f, "\tNONCE: NOTHING YET")?
        }

        Ok(())
    }
}

pub struct Blockchain<T> {
    pub tip: ByteBuf,
    pub db: Db,
    phantom: PhantomData<T>,
}

impl<T> Blockchain<T> {
    pub const DB_FILE: &'static str = "blockchain.kv";
    pub const BLOCKS_BUCKET: &'static str = "blocks";

    pub fn new() -> Self
    where
        T: Blockchainable,
    {
        let db = sled::open(Self::DB_FILE).expect("Could not open db file!");
        let blocks = db
            .open_tree(Self::BLOCKS_BUCKET)
            .expect("Could not open blocks bucket!");
        let last_hash = blocks.get(b"l").expect("Get value error!");
        let tip = if let Some(lh) = last_hash {
            ByteBuf::from(lh.to_vec())
        } else {
            let genesis_block = Block::new(T::genesis_data(), None);
            if let Some(hash) = &genesis_block.hash {
                blocks
                    .insert(
                        hash,
                        serialize(&genesis_block).expect("Serialization error!"),
                    )
                    .expect("Insertion error!");
                blocks
                    .insert(b"l", hash.to_vec())
                    .expect("Insertion error!");
                hash.clone()
            } else {
                ByteBuf::new()
            }
        };

        Blockchain {
            tip,
            db,
            phantom: PhantomData,
        }
    }

    pub fn add_block(&mut self, data: T)
    where
        T: Blockchainable,
    {
        let blocks = self
            .db
            .open_tree(Self::BLOCKS_BUCKET)
            .expect("Could not open blocks bucket!");
        let last_hash = blocks
            .get(b"l")
            .expect("Get value error!")
            .map(|v| ByteBuf::from(v.to_vec()));

        let new_block = Block::new(data, last_hash);
        if let Some(hash) = &new_block.hash {
            blocks
                .insert(hash, serialize(&new_block).expect("Serialization error"))
                .expect("Insertion error");
            blocks.insert(b"l", hash.to_vec()).expect("Insertion error");
            self.tip = hash.clone();
        }
    }

    pub fn remove_blocks(&self) {
        let blocks = self
            .db
            .open_tree(Self::BLOCKS_BUCKET)
            .expect("Could not open blocks bucket!");

        blocks.iter().for_each(|p| {
            if let Ok((key, _)) = p {
                blocks.remove(key).expect("Could not remove key!");
            }
        })
    }
}

impl<T> Iterator for Blockchain<T>
where
    T: Blockchainable,
{
    type Item = Block<T>;

    fn next(&mut self) -> Option<Self::Item> {
        let blocks = self
            .db
            .open_tree(Self::BLOCKS_BUCKET)
            .expect("Could not open blocks bucket!");
        if let Some(block) = blocks.get(&self.tip).expect("Get value error!") {
            let block = deserialize::<Block<T>>(&block).expect("Deserialization error!");
            if let Some(ph) = &block.previous_block_hash {
                self.tip = ph.clone();
            } else {
                self.tip = ByteBuf::new();
            }
            return Some(block);
        }
        None
    }
}

pub struct ProofOfWork<'a, T> {
    pub block: &'a Block<T>,
    pub target: BigUint,
}

impl<'a, T> ProofOfWork<'a, T>
where
    T: Blockchainable,
{
    const SHA_BITS: u64 = 256;
    const TARGET_BITS: u64 = 24;

    pub fn new(block: &'a Block<T>) -> Self {
        //target => 1[0...n] n=SHA_BITS-TARGET_BITS
        let mut target = BigUint::new(vec![1]);
        target = target << (Self::SHA_BITS - Self::TARGET_BITS);
        ProofOfWork { block, target }
    }

    fn prepare_data(&self, nonce: u64) -> ByteBuf {
        let mut buffer = ByteBuf::new();
        buffer.append(&mut serialize(self.block).expect("Serialization error!"));
        buffer.append(&mut Self::TARGET_BITS.to_be_bytes().to_vec());
        buffer.append(&mut nonce.to_be_bytes().to_vec());
        buffer
    }

    pub fn run(&self) -> Option<(u64, ByteBuf)> {
        println!("Mining for data: {}", self.block.data);
        let mut nonce = 0;
        while nonce < u64::MAX {
            let data = self.prepare_data(nonce);
            let hash = Sha256::new().chain_update(data).finalize();
            let hashint = BigUint::from_bytes_be(hash.as_slice());

            if hashint < self.target {
                let buffer = ByteBuf::from(hash.to_vec());
                return Some((nonce, buffer));
            }
            nonce += 1;
        }
        None
    }

    pub fn validate(&self) -> bool {
        if let Some(nonce) = self.block.nonce {
            let data = self.prepare_data(nonce);
            let hash = Sha256::new().chain_update(data).finalize();
            let hashint = BigUint::from_bytes_be(hash.as_slice());
            if hashint < self.target {
                return true;
            }
        }
        false
    }
}
