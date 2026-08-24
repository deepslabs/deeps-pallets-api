pub use secp256k1::{sign as secp_sign, Message, PublicKey, SecretKey};
use sp_runtime::traits::{IdentifyAccount, Verify};
use subxt::tx::Signer;
use subxt::Config;

/// A [`Signer`] implementation that can be constructed from an [`sp_core::Pair`].
#[derive(Clone, Debug)]
pub struct Secp256k1Signer<T: Config> {
    account_id: T::AccountId,
    signer: SecretKey,
}

impl<T> Secp256k1Signer<T>
where
    T: Config,
    T::Signature: Verify,
    <T::Signature as Verify>::Signer:
        From<sp_core::ecdsa::Public> + IdentifyAccount<AccountId = T::AccountId>,
{
    /// Creates a new [`Signer`] for evm ecdsa
    pub fn new(signer: SecretKey) -> Self {
        let pk_compressed = PublicKey::from_secret_key(&signer).serialize_compressed();
        let account_id =
            <T::Signature as Verify>::Signer::from(sp_core::ecdsa::Public::from_raw(pk_compressed))
                .into_account();
        Self {
            account_id: account_id.into(),
            signer,
        }
    }

    /// Returns the [`sp_core::Pair`] implementation used to construct this.
    pub fn signer(&self) -> &SecretKey {
        &self.signer
    }

    /// Return the account ID.
    pub fn account_id(&self) -> &T::AccountId {
        &self.account_id
    }
}

impl<T> Signer<T> for Secp256k1Signer<T>
where
    T: Config,
    T::Signature: From<sp_runtime::MultiSignature>,
    T::AccountId: Into<[u8; 20]>,
    <T as Config>::Address: From<T::AccountId>,
    T::Signature: Verify,
{
    fn account_id(&self) -> &T::AccountId {
        &self.account_id
    }

    fn address(&self) -> T::Address {
        T::Address::from(self.account_id.clone())
    }

    fn sign(&self, signer_payload: &[u8]) -> T::Signature {
        let msg = Message::parse(&sp_core::keccak_256(signer_payload));
        let signature = secp_sign(&msg, &self.signer);
        let mut sig = [0u8; 65];
        sig[..64].copy_from_slice(signature.0.serialize().as_slice());
        sig[64] = signature.1.serialize();
        let ecdsa_signature = sp_core::ecdsa::Signature::from_raw(sig);
        let multi_sig: sp_runtime::MultiSignature = ecdsa_signature.into();
        multi_sig.into()
    }
}
