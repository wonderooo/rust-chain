use p256::{
    ecdsa::{SigningKey, VerifyingKey},
    elliptic_curve::rand_core::OsRng,
};
use ripemd::Ripemd160;
use serde_bytes::ByteBuf;
use sha2::{Digest, Sha256};

pub struct Wallet {
    pub public_key: VerifyingKey,
    pub private_key: SigningKey,
}

impl Wallet {
    const VERSION: [u8; 1] = [0x0];
    const CHECKSUM_LEN: usize = 4;

    pub fn new() -> Self {
        let private_key = SigningKey::random(&mut OsRng);
        let public_key = VerifyingKey::from(&private_key);
        Self {
            public_key,
            private_key,
        }
    }

    pub fn address(&self) -> ByteBuf {
        let sha_public = Sha256::new()
            .chain_update(&self.public_key.to_encoded_point(false))
            .finalize();
        let ripemd_public = Ripemd160::new().chain_update(&sha_public).finalize();

        let mut versioned = ByteBuf::from(Self::VERSION);
        versioned.append(&mut ripemd_public.to_vec());

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
}
