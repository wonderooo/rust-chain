use std::fmt::Display;

use bincode::serialize;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use sha2::{Digest, Sha256};

use crate::{blockchain::Blockchain, Blockchainable};

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
            script_sig: data.clone(),
        };

        let txout = TXOutput {
            value: Self::SUBSIDY,
            script_pk: to.clone(),
        };

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

        let (all, valid_outputs) = blockchain.find_spendable_outputs(from, value);
        if all < value {
            panic!("Not enough coins!")
        }

        for (txid, out_idx) in valid_outputs {
            out_idx.iter()
                .for_each(|idx| vin.push(TXInput { txid: txid.clone(), vout: Some(*idx), script_sig: from.clone() }))
        }

        vout.push(TXOutput { value, script_pk: to.clone() });
        if all > value {
            vout.push(TXOutput { value: all - value, script_pk: from.clone() })
        }

        let mut tx = Self { id: ByteBuf::new(), vin, vout};
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
    /// Provides data to be used in script_pk
    pub script_sig: String,
}

impl TXInput {
    pub fn can_unlock_output_with(&self, unlocking_data: &String) -> bool {
        self.script_sig == *unlocking_data
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct TXOutput {
    /// Like quantity of coins in the outputting tx
    pub value: u64,
    /// "puzzle" to unlock coins in tx
    pub script_pk: String,
}

impl TXOutput {
    pub fn can_be_unlocked_with(&self, unlocking_data: &String) -> bool {
        self.script_pk == *unlocking_data
    }
}
