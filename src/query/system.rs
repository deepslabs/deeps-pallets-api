use subxt::utils::H256 as Hash;

pub struct System<'a> {
    pub(crate) client: &'a crate::NodeClient,
}

impl<'a> System<'a> {
    pub async fn block_hash(
        &self,
        height: u32,
        at_block: Option<Hash>,
    ) -> Result<Option<Hash>, subxt::Error> {
        let storage_query = crate::node::storage().system().block_hash(height);
        self.client.query_storage(storage_query, at_block).await
    }
}
