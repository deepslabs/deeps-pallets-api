use crate::node::runtime_types::pallet_committee_health::pallet::{
    ConfirmData, ConsensusStage, DHCState,
};
use crate::node::runtime_types::pallet_mining::types::DeviceMode;
use subxt::utils::H256 as Hash;

pub struct CommitteeHealth<'a> {
    pub(crate) client: &'a crate::NodeClient,
}

impl<'a> CommitteeHealth<'a> {
    pub async fn identity_challenge(
        &self,
        identity: Vec<u8>,
        at_block: Option<Hash>,
    ) -> Result<(u32, Vec<u8>), subxt::Error> {
        let store = crate::node::storage()
            .committee_health()
            .identity_challenge(identity);
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or_default())
    }

    pub async fn court_members(
        &self,
        device_mode: DeviceMode,
        at_block: Option<Hash>,
    ) -> Result<Option<Vec<Vec<u8>>>, subxt::Error> {
        let store = crate::node::storage()
            .committee_health()
            .court_members(device_mode);
        self.client.query_storage(store, at_block).await
    }

    pub async fn consensus_state(
        &self,
        device_mode: DeviceMode,
        at_block: Option<Hash>,
    ) -> Result<Option<DHCState<Hash>>, subxt::Error> {
        let store = crate::node::storage()
            .committee_health()
            .consensus_state(device_mode);
        self.client.query_storage(store, at_block).await
    }

    pub async fn state_votes(
        &self,
        device_mode: DeviceMode,
        device_id: Vec<u8>,
        at_block: Option<Hash>,
    ) -> Result<Vec<u8>, subxt::Error> {
        let store = crate::node::storage()
            .committee_health()
            .state_votes(device_mode, device_id);
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or_default())
    }

    pub async fn consensus_confirms(
        &self,
        device_mode: DeviceMode,
        stage: ConsensusStage,
        at_block: Option<Hash>,
    ) -> Result<Option<ConfirmData<Hash>>, subxt::Error> {
        let store = crate::node::storage()
            .committee_health()
            .consensus_confirms(device_mode, stage);
        self.client.query_storage(store, at_block).await
    }

    pub async fn submit_device_whitelist(
        &self,
        device_mode: DeviceMode,
        at_block: Option<Hash>,
    ) -> Result<Vec<Vec<u8>>, subxt::Error> {
        let store = crate::node::storage()
            .committee_health()
            .submit_device_whitelist(device_mode);
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or_default())
    }

    pub async fn submit_devices(
        &self,
        device_mode: DeviceMode,
        at_block: Option<Hash>,
    ) -> Result<Vec<Vec<u8>>, subxt::Error> {
        let store = crate::node::storage()
            .committee_health()
            .submit_devices(device_mode);
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or_default())
    }

    pub async fn submit_devices_size(
        &self,
        device_mode: DeviceMode,
        at_block: Option<Hash>,
    ) -> Result<u16, subxt::Error> {
        let store = crate::node::storage()
            .committee_health()
            .submit_devices_size(device_mode);
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or_default())
    }
}
