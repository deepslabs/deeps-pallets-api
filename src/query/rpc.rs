use crate::node::runtime_types::fp_account::AccountId20 as RuntimeAccountId20;
use crate::node::runtime_types::pallet_rpc::pallet::DeviceInfo;
use subxt::utils::H256 as Hash;

pub struct Rpc<'a> {
    pub(crate) client: &'a crate::NodeClient,
}

impl<'a> Rpc<'a> {
    pub async fn device_info_rpc(
        &self,
        id: Vec<u8>,
        at_block: Option<Hash>,
    ) -> Result<Option<DeviceInfo<RuntimeAccountId20, u32>>, subxt::Error> {
        let storage_query = crate::node::storage().rpc().devices(id.clone());
        self.client.query_storage(storage_query, at_block).await
    }

    pub async fn relate_deviceid_rpc(
        &self,
        id: Vec<u8>,
        at_block: Option<Hash>,
    ) -> Result<Option<Vec<Vec<u8>>>, subxt::Error> {
        let storage_query = crate::node::storage()
            .rpc()
            .watcher_deviceid_map_rpc_deviceid(id.clone());
        self.client.query_storage(storage_query, at_block).await
    }

    pub async fn eth_checkpoint(
        &self,
        at_block: Option<Hash>,
    ) -> Result<Option<Vec<u8>>, subxt::Error> {
        let storage_query = crate::node::storage().rpc().eth_checkpoint();
        self.client.query_storage(storage_query, at_block).await
    }
}
