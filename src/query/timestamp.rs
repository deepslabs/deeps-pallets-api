use subxt::utils::H256 as Hash;

pub struct Timestamp<'a> {
    pub(crate) client: &'a crate::NodeClient,
}

impl<'a> Timestamp<'a> {
    pub async fn now(&self, at_block: Option<Hash>) -> Result<Option<u64>, subxt::Error> {
        let storage_query = crate::node::storage().timestamp().now();
        self.client.query_storage(storage_query, at_block).await
    }
}
