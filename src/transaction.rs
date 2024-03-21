use std::fmt::Display;

use bincode::serialize;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use sha2::{Digest, Sha256};

use crate::{blockchain::Blockchain, wallet::{Wallet, Wallets}, Blockchainable};

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Transaction {
    /// Tx id in for of bytes hash
    pub id: ByteBuf,
    /// Inputs that participate in tx
    pub vin: Vec<TXInput>,
    /// Outputs that participate in tx
    pub vout: Vec<TXOutput>,
}

impl Transaction {
    const SUBSIDY: u64 = 10;

    pub fn new_coinbase_tx(to: &String, data: &String) -> Self {
        let txin = TXInput {
            txid: ByteBuf::new(),
            vout: None,
            signature: ByteBuf::new(),
            pub_key: ByteBuf::from(data.clone())
        };

        let mut txout = TXOutput {
            value: Self::SUBSIDY,
            pub_key_hash: ByteBuf::new(),
        };
        txout.lock(&ByteBuf::from(to.clone()));

        let mut tx = Transaction {
            id: ByteBuf::new(),
            vin: vec![txin],
            vout: vec![txout],
        };
        tx.set_id();
        tx
    }

    pub fn new_tx<T>(to: &String, from: &String, value: u64, blockchain: &mut Blockchain<T>) -> Self
    where
        T: Blockchainable,
    {
        let mut vin = Vec::new();
        let mut vout = Vec::new();

        let wallets = Wallets::fetch_wallets();
        let wallet = wallets.get(&ByteBuf::from(from.clone())).expect("Wallet with address not found!");
        let pub_key_hash = Wallet::hash_pub_key(&wallet.public_key);
        let (all, valid_outputs) = blockchain.find_spendable_outputs(&pub_key_hash, value);
        if all < value {
            panic!("Not enough coins!")
        }

        for (txid, out_idx) in valid_outputs {
            out_idx.iter().for_each(|idx| {
                vin.push(TXInput {
                    txid: txid.clone(),
                    vout: Some(*idx),
                    signature: ByteBuf::new(),
                    pub_key: wallet.public_key.clone(),
                })
            })
        }

        let mut txout_th = TXOutput { value, pub_key_hash: ByteBuf::new() };
        txout_th.lock(&ByteBuf::from(to.clone()));
        vout.push(txout_th);

        if all > value {
            let mut txout_rest = TXOutput { value: all - value, pub_key_hash: ByteBuf::new() };
            txout_rest.lock(&ByteBuf::from(from.clone()));
            vout.push(txout_rest);
        }

        let mut tx = Self {
            id: ByteBuf::new(),
            vin,
            vout,
        };
        tx.set_id();
        tx
    }

    fn set_id(&mut self) {
        let serialized = serialize(&self).expect("Serialization error!");
        let hash = Sha256::new().chain_update(serialized).finalize();
        self.id = ByteBuf::from(hash.to_vec());
    }

    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].txid.len() == 0 && self.vin[0].vout.is_none()
    }
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ID: {}, ", hex::encode(self.id.clone()))?;
        write!(f, "LEN VIN: {}, ", self.vin.len())?;
        write!(f, "LEN VOUT: {}", self.vout.len())?;
        Ok(())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TXInput {
    /// Id of tx that this input connects output
    pub txid: ByteBuf,
    /// Index of output reference in connected tx
    pub vout: Option<usize>,
    /// TODO
    pub signature: ByteBuf,
    /// TODO
    pub pub_key: ByteBuf,
}

impl TXInput {
    pub fn uses_key(&self, pub_key_hash: &ByteBuf) -> bool {
        let locking_hash = Wallet::hash_pub_key(&self.pub_key);
        locking_hash == *pub_key_hash
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TXOutput {
    /// Like quantity of coins in the outputting tx
    pub value: u64,
    /// TODO
    pub pub_key_hash: ByteBuf,
}

impl TXOutput {
    pub fn is_locked_with(&self, pub_key_hash: &ByteBuf) -> bool {
        self.pub_key_hash == *pub_key_hash
    }

    pub fn lock(&mut self, address: &ByteBuf) {
        let pub_key_hash = bs58::decode(address)
            .with_alphabet(bs58::Alphabet::BITCOIN)
            .into_vec()
            .expect("Address could not be decoded to base58!");
        let pub_key_hash =
            ByteBuf::from(&pub_key_hash[1..pub_key_hash.len() - Wallet::CHECKSUM_LEN]);

        self.pub_key_hash = pub_key_hash;
    }
}
