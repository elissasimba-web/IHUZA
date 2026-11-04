#![no_std]

use soroban_sdk::{contract, contracterror, contractimpl, contracttype, Address, BytesN, Env};

#[contracttype]
enum Key {
    Used(Address, BytesN<32>),
}

#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq)]
#[repr(u32)]
pub enum Error {
    Expired = 1,
    AlreadyUsed = 2,
    ExpiryTooFar = 3,
}

/// Records one wallet-authorized, time-limited intent commitment.
/// It never moves assets or declares a payment successful.
#[contract]
pub struct IntentAnchor;

#[contractimpl]
impl IntentAnchor {
    pub fn authorize(
        env: Env,
        owner: Address,
        intent_hash: BytesN<32>,
        expires_at: u64,
    ) -> Result<(), Error> {
        if expires_at <= env.ledger().timestamp() {
            return Err(Error::Expired);
        }
        if expires_at > env.ledger().timestamp().saturating_add(300) {
            return Err(Error::ExpiryTooFar);
        }

        owner.require_auth();
        let key = Key::Used(owner, intent_hash);
        let storage = env.storage().persistent();
        if storage.has(&key) {
            return Err(Error::AlreadyUsed);
        }
        storage.set(&key, &expires_at);
        Ok(())
    }

    pub fn authorized(env: Env, owner: Address, intent_hash: BytesN<32>) -> bool {
        let key = Key::Used(owner, intent_hash);
        let expiry: Option<u64> = env.storage().persistent().get(&key);
        match expiry {
            Some(timestamp) => env.ledger().timestamp() < timestamp,
            None => false,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use soroban_sdk::testutils::{Address as _, Ledger};

    #[test]
    fn owner_authorizes_once_until_expiry() {
        let env = Env::default();
        env.ledger().with_mut(|li| li.timestamp = 100);
        let contract = env.register(IntentAnchor, ());
        let client = IntentAnchorClient::new(&env, &contract);
        let owner = Address::generate(&env);
        let hash = BytesN::from_array(&env, &[7; 32]);

        assert!(!client.authorized(&owner, &hash));
        assert!(client.try_authorize(&owner, &hash, &200).is_err());
        assert!(!client.authorized(&owner, &hash));
        client.mock_all_auths().authorize(&owner, &hash, &200);
        assert!(client.authorized(&owner, &hash));
        assert_eq!(
            client.mock_all_auths().try_authorize(&owner, &hash, &200),
            Err(Ok(Error::AlreadyUsed))
        );

        let other_hash = BytesN::from_array(&env, &[8; 32]);
        assert_eq!(
            client.try_authorize(&owner, &other_hash, &500),
            Err(Ok(Error::ExpiryTooFar))
        );

        env.ledger().with_mut(|li| li.timestamp = 200);
        assert!(!client.authorized(&owner, &hash));
        assert_eq!(
            client.try_authorize(&owner, &hash, &200),
            Err(Ok(Error::Expired))
        );
    }
}
