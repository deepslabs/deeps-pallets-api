use subxt::utils::H256 as Hash;

pub struct Facility<'a> {
    pub(crate) client: &'a crate::NodeClient,
}

impl<'a> Facility<'a> {
    pub async fn hash_to_version(
        &self,
        version: u16,
        at_block: Option<Hash>,
    ) -> Result<Option<Vec<u8>>, subxt::Error> {
        let store = crate::node::storage().facility().hash_to_version(&version);
        self.client.query_storage(store, at_block).await
    }

    pub async fn version_list(&self, at_block: Option<Hash>) -> Result<Vec<u16>, subxt::Error> {
        let store = crate::node::storage().facility().version_list();
        self.client.query_storage_or_default(store, at_block).await
    }
}
