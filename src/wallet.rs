use std::{
    collections::HashMap,
    fmt::Display,
    fs::File,
    io::{Read, Write},
};

use bincode::{deserialize, serialize};
use p256::{
    ecdsa::{SigningKey, VerifyingKey},
    elliptic_curve::rand_core::OsRng,
};
use ripemd::Ripemd160;
use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use sha2::{Digest, Sha256};

const WALLETS_FILE: &'static str = "wallets.dat";

#[derive(Serialize, Deserialize, Clone)]
pub struct Wallet {
    #[serde(with = "serde_bytes")]
    pub public_key: ByteBuf,
    #[serde(with = "serde_bytes")]
    pub private_key: ByteBuf,
}

impl Wallet {
    pub const VERSION: [u8; 1] = [0x0];
    pub const CHECKSUM_LEN: usize = 4;

    pub fn new() -> Self {
        let private_key = SigningKey::random(&mut OsRng);
        let public_key = VerifyingKey::from(&private_key);
        Self {
            public_key: ByteBuf::from(public_key.to_encoded_point(false).to_bytes()),
            private_key: ByteBuf::from(private_key.to_bytes().to_vec()),
        }
    }

    pub fn address(&self) -> ByteBuf {
        let hash_pub = Self::hash_pub_key(&self.public_key);

        let mut versioned = ByteBuf::from(Self::VERSION);
        versioned.append(&mut hash_pub.to_vec());

        let checksum = Sha256::new()
            .chain_update(&Sha256::new().chain_update(&versioned).finalize())
            .finalize();

        versioned.append(&mut checksum[..Self::CHECKSUM_LEN].to_vec());
        ByteBuf::from(
            bs58::encode(versioned)
                .with_alphabet(bs58::Alphabet::BITCOIN)
                .into_vec(),
        )
    }

    pub fn hash_pub_key(public_key: &ByteBuf) -> ByteBuf {
        let sha_public = Sha256::new().chain_update(&public_key).finalize();
        let ripemd_public = Ripemd160::new().chain_update(&sha_public).finalize();
        ByteBuf::from(ripemd_public.to_vec())
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct Wallets(HashMap<ByteBuf, Wallet>);

impl Wallets {
    pub fn save_wallet(wallet: &Wallet) {
        let mut wallets = Self::fetch_wallets();

        let wallet_addr = wallet.address();
        wallets.0.insert(wallet_addr, wallet.clone());

        let mut file = File::create(WALLETS_FILE).expect("Open file error!");
        file.write_all(&serialize(&wallets).expect("Serialization error!"))
            .expect("File write error!");
        file.flush().expect("File flush error!");
    }

    pub fn fetch_wallets() -> Self {
        if let Ok(ref mut file) = File::open(WALLETS_FILE) {
            let mut buffer = Vec::new();
            file.read_to_end(&mut buffer).expect("Read to end error!");
            deserialize::<Wallets>(&buffer).expect("Deserialization error!")
        } else {
            Wallets(HashMap::new())
        }
    }

    pub fn get(&self, address: &ByteBuf) -> Option<&Wallet> {
        self.0.get(address)
    }
}

impl Display for Wallets {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (addr, wallet) in &self.0 {
            writeln!(f, "WALLET:")?;
            write!(
                f,
                "\tADDR: {}, ",
                std::str::from_utf8(&addr).expect("Could not convert bytes to string!")
            )?;
            write!(
                f,
                "PUB: {}..., ",
                hex::encode(&wallet.public_key)
                    .chars()
                    .take(40)
                    .collect::<String>()
            )?;
            writeln!(
                f,
                "PRIV: {}...",
                hex::encode(&wallet.private_key)
                    .chars()
                    .take(40)
                    .collect::<String>()
            )?;
        }
        Ok(())
    }
}
