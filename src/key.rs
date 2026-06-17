//! ED25519 key generation.

use ed25519_dalek::SigningKey;
use rand::RngExt;

/// ED25519 public key: 32 bytes.
pub type PublicKey = [u8; 32];
/// ED25519 private key (expanded form): 64 bytes = seed(32) followed by public_key(32).
pub type PrivateKey = [u8; 64];

/// ED25519 key pair (raw bytes, can be written directly to files).
#[derive(Debug, Clone)]
pub struct Ed25519KeyPair {
    pub public_key: PublicKey,
    pub private_key: PrivateKey,
}

/// Generate a random ED25519 key pair.
///
/// Uses the system CSPRNG to generate a 32-byte seed and derive the public key.
///
/// # Example
///
/// ```ignore
/// let kp = machikado_rs::generate_keypair();
/// std::fs::write("public_key", kp.public_key)?;
/// std::fs::write("private_key", kp.private_key)?;
/// ```
pub fn generate_keypair() -> Ed25519KeyPair {
    let mut rng = rand::rng();
    let seed: [u8; 32] = rng.random();
    let signing_key = SigningKey::from_bytes(&seed);
    let public_key = signing_key.verifying_key().to_bytes();
    let private_key = signing_key.to_keypair_bytes();
    Ed25519KeyPair {
        public_key,
        private_key,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_keypair() {
        let kp1 = generate_keypair();
        let kp2 = generate_keypair();
        // Two generations should produce different keys
        assert_ne!(kp1.public_key, kp2.public_key);
        assert_ne!(kp1.private_key, kp2.private_key);
        // The first 32 bytes (seed) of the private key should derive the public key
        let signing_key = SigningKey::from_bytes(&kp1.private_key[..32].try_into().unwrap());
        assert_eq!(signing_key.verifying_key().to_bytes(), kp1.public_key);
        // Last 32 bytes of the expanded private key equal the public key
        assert_eq!(&kp1.private_key[32..], &kp1.public_key);
    }
}
