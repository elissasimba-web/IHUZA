use base64::{engine::general_purpose::STANDARD, Engine};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};
use stellar_strkey::ed25519::PublicKey;

pub const TESTNET_PASSPHRASE: &str = "Test SDF Network ; September 2015";
const SEP53_PREFIX: &[u8] = b"Stellar Signed Message:\n";
const MAX_LIFETIME_SECONDS: u64 = 300;

/// A server-issued challenge. The caller must generate a random nonce, store it
/// against an authenticated IHUZA+ account, and consume it atomically on success.
pub struct Challenge {
    pub address: String,
    pub nonce_hex: String,
    pub issued_at: u64,
    pub expires_at: u64,
}

impl Challenge {
    pub fn message(&self) -> String {
        format!(
            "IHUZA+ wallet link\nversion: 1\nnetwork: {TESTNET_PASSPHRASE}\naddress: {}\nnonce: {}\nissued_at: {}\nexpires_at: {}",
            self.address, self.nonce_hex, self.issued_at, self.expires_at
        )
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum ProofError {
    InvalidChallenge,
    Expired,
    InvalidSignature,
}

/// Verifies SEP-53's SHA-256(prefix || UTF-8 message) Ed25519 signature.
/// This proves possession of the key, not full Stellar multisig account control.
pub fn verify_sep53(address: &str, message: &str, signature_base64: &str) -> bool {
    let Ok(public_key) = PublicKey::from_string(address) else {
        return false;
    };
    let Ok(verifying_key) = VerifyingKey::from_bytes(&public_key.0) else {
        return false;
    };
    let Ok(signature_bytes) = STANDARD.decode(signature_base64) else {
        return false;
    };
    let Ok(signature) = Signature::from_slice(&signature_bytes) else {
        return false;
    };
    let digest = Sha256::new()
        .chain_update(SEP53_PREFIX)
        .chain_update(message.as_bytes())
        .finalize();
    verifying_key.verify(&digest, &signature).is_ok()
}

/// Cryptographic and time check only. A successful result must not create a
/// session or bind an account until the server atomically consumes the nonce.
pub fn verify_challenge(
    challenge: &Challenge,
    now: u64,
    signature_base64: &str,
) -> Result<(), ProofError> {
    let nonce_valid = challenge.nonce_hex.len() == 64
        && challenge.nonce_hex.bytes().all(|b| b.is_ascii_hexdigit());
    let lifetime_valid = challenge.issued_at > 0
        && challenge.expires_at > challenge.issued_at
        && challenge.expires_at - challenge.issued_at <= MAX_LIFETIME_SECONDS;
    if !nonce_valid || !lifetime_valid {
        return Err(ProofError::InvalidChallenge);
    }
    if now < challenge.issued_at || now >= challenge.expires_at {
        return Err(ProofError::Expired);
    }
    if !verify_sep53(&challenge.address, &challenge.message(), signature_base64) {
        return Err(ProofError::InvalidSignature);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    // Public test vector from SEP-53; no private key is stored here.
    const ADDRESS: &str = "GBXFXNDLV4LSWA4VB7YIL5GBD7BVNR22SGBTDKMO2SBZZHDXSKZYCP7L";
    const SIGNATURE: &str =
        "fO5dbYhXUhBMhe6kId/cuVq/AfEnHRHEvsP8vXh03M1uLpi5e46yO2Q8rEBzu3feXQewcQE5GArp88u6ePK6BA==";

    #[test]
    fn verifies_official_sep53_vector() {
        assert!(verify_sep53(ADDRESS, "Hello, World!", SIGNATURE));
        assert!(!verify_sep53(ADDRESS, "Hello, World?", SIGNATURE));
        assert!(!verify_sep53("not-an-address", "Hello, World!", SIGNATURE));
    }

    #[test]
    fn rejects_stale_or_malformed_challenges_before_signature_check() {
        let challenge = Challenge {
            address: ADDRESS.to_owned(),
            nonce_hex: "a".repeat(64),
            issued_at: 100,
            expires_at: 400,
        };
        assert_eq!(
            verify_challenge(&challenge, 400, "bad"),
            Err(ProofError::Expired)
        );
        assert_eq!(
            verify_challenge(&challenge, 99, "bad"),
            Err(ProofError::Expired)
        );
        assert_eq!(
            verify_challenge(&challenge, 200, "bad"),
            Err(ProofError::InvalidSignature)
        );
        let malformed = Challenge {
            nonce_hex: "short".to_owned(),
            ..challenge
        };
        assert_eq!(
            verify_challenge(&malformed, 200, "bad"),
            Err(ProofError::InvalidChallenge)
        );
    }
}
