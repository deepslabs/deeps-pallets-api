use crate::node::runtime_types::ethereum::transaction::TransactionV3 as Transaction;
use crate::handle_custom_error;
use subxt::utils::H256 as Hash;

pub struct Ethereum<'a> {
    pub(crate) client: &'a crate::NodeClient,
}

impl<'a> Ethereum<'a> {
    pub async fn transact(&self, transaction: Transaction) -> Result<Hash, String> {
        let call = crate::node::tx().ethereum().transact(transaction);
        self.client
            .submit_extrinsic_without_signer(call)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn transact_unsigned(&self, transaction: Transaction) -> Result<Hash, String> {
        let call = crate::node::tx().ethereum().transact_unsigned(transaction);
        self.client
            .submit_extrinsic_without_signer(call)
            .await
            .map_err(handle_custom_error)
    }

    pub async fn transact_unsigned_call_bytes(
        &self,
        transaction: Transaction,
    ) -> Result<Vec<u8>, String> {
        let call = crate::node::tx().ethereum().transact_unsigned(transaction);
        self.client
            .unsigned_tx_encode_to_bytes(call)
            .await
            .map_err(|e| e.to_string())
    }
}
