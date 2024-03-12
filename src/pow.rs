use bincode::serialize;
use num_bigint::BigUint;
use serde_bytes::ByteBuf;
use sha2::{Sha256, Digest};

use crate::{block::Block, Blockchainable};

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