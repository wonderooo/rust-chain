use std::marker::PhantomData;

use bincode::{deserialize, serialize};
use serde_bytes::ByteBuf;
use sled::Db;

use crate::{block::Block, Blockchainable};

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