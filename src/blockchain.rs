use std::{collections::HashMap, marker::PhantomData};

use bincode::{deserialize, serialize};
use serde_bytes::ByteBuf;
use sled::Db;

use crate::{
    block::Block,
    transaction::{TXOutput, Transaction},
    Blockchainable,
};

pub struct Blockchain<T> {
    pub tip: ByteBuf,
    pub db: Db,
    phantom: PhantomData<T>,
}

impl<T> Blockchain<T> {
    pub const DB_FILE: &'static str = "blockchain.kv";
    pub const BLOCKS_BUCKET: &'static str = "blocks";
    const GENESIS_COINBASE: &'static str =
        "The Times 03/Jan/2009 Chancellor on brink of second bailout for banks";

    pub fn new(address: &String) -> Self
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
            let genesis_block = Block::<T>::new(
                vec![Transaction::new_coinbase_tx(
                    &address,
                    &Self::GENESIS_COINBASE.to_string(),
                )],
                None,
            );
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

    pub fn add_block(&mut self, data: Vec<Transaction>)
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

        let new_block = Block::<T>::new(data, last_hash);
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

    pub fn find_unspent_txs(&mut self, address: &String) -> Vec<Transaction>
    where
        T: Blockchainable,
    {
        let mut spent_txos: HashMap<ByteBuf, Vec<usize>> = HashMap::new();
        let mut unspent_txs: Vec<Transaction> = Vec::new();

        for block in self {
            for tx in block.transactions {
                'outs: for (this_idx, vout) in tx.vout.iter().enumerate() {
                    if let Some(indicies) = spent_txos.get(&tx.id) {
                        for idx in indicies {
                            if *idx == this_idx {
                                continue 'outs;
                            }
                        }
                    }

                    if vout.can_be_unlocked_with(address) {
                        unspent_txs.push(tx.clone());
                    }
                }

                if !tx.is_coinbase() {
                    for vin in &tx.vin {
                        if vin.can_unlock_output_with(address) {
                            if let Some(indicies) = spent_txos.get_mut(&vin.txid) {
                                indicies.push(vin.vout.expect("How tf!"));
                            } else {
                                spent_txos
                                    .insert(vin.txid.clone(), vec![vin.vout.expect("How tf!")]);
                            }
                        }
                    }
                }

                if let Some(ref ph) = block.previous_block_hash {
                    if ph.len() == 0 {
                        break;
                    }
                }
            }
        }

        unspent_txs
    }

    pub fn find_utxo(&mut self, address: &String) -> Vec<TXOutput>
    where
        T: Blockchainable,
    {
        self.find_unspent_txs(address)
            .iter()
            .flat_map(|utx| utx.vout.clone())
            .filter(|txo| txo.can_be_unlocked_with(address))
            .collect()
    }

    pub fn balance_at(&mut self, address: &String) -> u64
    where
        T: Blockchainable,
    {
        let f = self.find_utxo(address);
        f.iter().fold(0, |acc, utxo| utxo.value + acc)
    }

    pub fn send(&mut self, from: &String, to: &String, value: u64)
    where
        T: Blockchainable,
    {
        let tx = Transaction::new_tx(to, from, value, self);
        self.add_block(vec![tx]);
    }

    pub fn find_spendable_outputs(
        &mut self,
        address: &String,
        value: u64,
    ) -> (u64, HashMap<ByteBuf, Vec<usize>>)
    where
        T: Blockchainable,
    {
        let mut unspent_outputs: HashMap<ByteBuf, Vec<usize>> = HashMap::new();
        let unspent_tx = self.find_unspent_txs(address);
        let mut all = 0;

        'outer: for tx in unspent_tx {
            for (idx, vout) in tx.vout.iter().enumerate() {
                if vout.can_be_unlocked_with(address) && all < value {
                    all += vout.value;
                    if let Some(indicies) = unspent_outputs.get_mut(&tx.id) {
                        indicies.push(idx);
                    } else {
                        unspent_outputs.insert(tx.id.clone(), vec![idx]);
                    }

                    if all >= value {
                        break 'outer;
                    }
                }
            }
        }

        (all, unspent_outputs)
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
