use std::{fmt::Display, time::{SystemTime, UNIX_EPOCH}};

use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;

use crate::{pow::ProofOfWork, Blockchainable};

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