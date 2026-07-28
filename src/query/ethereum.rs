use subxt::utils::H256 as Hash;

pub struct Ethereum<'a> {
    pub(crate) client: &'a crate::NodeClient,
}

impl<'a> Ethereum<'a> {
    pub async fn evm_chain_id(&self, at_block: Option<Hash>) -> Result<Option<u64>, subxt::Error> {
        let store = crate::node::storage().evm_chain_id().chain_id();
        self.client.query_storage(store, at_block).await
    }
}
