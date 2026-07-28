use subxt::utils::H256 as Hash;

pub struct Configs<'a> {
    pub(crate) client: &'a crate::NodeClient,
}

impl<'a> Configs<'a> {
    pub async fn round_msg_wait(
        &self,
        at_block: Option<Hash>,
    ) -> Result<Option<u64>, subxt::Error> {
        let store = crate::node::storage().configs().round_msg_wait();
        self.client.query_storage(store, at_block).await
    }

    pub async fn round_msg_request_limit(
        &self,
        at_block: Option<Hash>,
    ) -> Result<Option<u8>, subxt::Error> {
        let store = crate::node::storage().configs().round_msg_request_limit();
        self.client.query_storage(store, at_block).await
    }

    pub async fn monitor_delay_tolerance(
        &self,
        chain_id: u32,
        at_block: Option<Hash>,
    ) -> Result<u64, subxt::Error> {
        let store = crate::node::storage()
            .configs()
            .monitor_delay_tolerance(chain_id);
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or_default())
    }

    pub async fn monitor_delay_tolerance_iter(
        &self,
        at_block: Option<Hash>,
    ) -> Result<Vec<(u32, u64)>, subxt::Error> {
        let store = crate::node::storage()
            .configs()
            .monitor_delay_tolerance_root();
        self.client
            .query_storage_value_iter(store, 300, at_block)
            .await
            .map(|res| {
                res.into_iter()
                    .map(|(key, v)| {
                        let mut cid_bytes = [0u8; 4];
                        cid_bytes.copy_from_slice(&key.0[48..]);
                        (u32::from_le_bytes(cid_bytes), v)
                    })
                    .collect()
            })
    }

    pub async fn device_url_map(
        &self,
        id: Vec<u8>,
        at_block: Option<Hash>,
    ) -> Result<Option<Vec<u8>>, subxt::Error> {
        let storage_query = crate::node::storage().configs().device_url_map(id);
        self.client.query_storage(storage_query, at_block).await
    }

    pub async fn simple_sign(&self, at_block: Option<Hash>) -> Result<bool, subxt::Error> {
        let storage_query = crate::node::storage().configs().simple_sign();
        self.client
            .query_storage(storage_query, at_block)
            .await
            .map(|r| r.unwrap_or_default())
    }

    pub async fn simple_key(&self, at_block: Option<Hash>) -> Result<bool, subxt::Error> {
        let storage_query = crate::node::storage().configs().simple_key();
        self.client
            .query_storage(storage_query, at_block)
            .await
            .map(|r| r.unwrap_or_default())
    }

    pub async fn device_heartbeat_interval(
        &self,
        at_block: Option<Hash>,
    ) -> Result<Option<u64>, subxt::Error> {
        let store = crate::node::storage().configs().device_heartbeat_interval();
        self.client.query_storage(store, at_block).await
    }
}
