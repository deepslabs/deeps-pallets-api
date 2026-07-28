use crate::node::runtime_types::fp_account::AccountId20;
use crate::node::runtime_types::pallet_committee::types::{
    Committee as CommitteeP, GlobalConfig, MissionType,
};
use crate::node::runtime_types::pallet_mining::types::DeviceMode;
use subxt::utils::H256 as Hash;

pub struct Committee<'a> {
    pub(crate) client: &'a crate::NodeClient,
}

impl<'a> Committee<'a> {
    pub async fn global_epoch(&self, at_block: Option<Hash>) -> Result<u64, subxt::Error> {
        let store = crate::node::storage().committee().global_epoch();
        self.client.query_storage_or_default(store, at_block).await
    }

    pub async fn epoch_config(
        &self,
        at_block: Option<Hash>,
    ) -> Result<GlobalConfig<u32>, subxt::Error> {
        let store = crate::node::storage().committee().epoch_config();
        self.client.query_storage_or_default(store, at_block).await
    }

    pub async fn next_epoch_config(
        &self,
        at_block: Option<Hash>,
    ) -> Result<Option<GlobalConfig<u32>>, subxt::Error> {
        let store = crate::node::storage().committee().next_epoch_config();
        self.client.query_storage(store, at_block).await
    }

    pub async fn pool_rate(&self, at_block: Option<Hash>) -> Result<u8, subxt::Error> {
        let store = crate::node::storage().committee().pool_rate();
        self.client.query_storage_or_default(store, at_block).await
    }

    pub async fn committees(
        &self,
        cid: u32,
        at_block: Option<Hash>,
    ) -> Result<Option<CommitteeP<AccountId20, u32>>, subxt::Error> {
        let store = crate::node::storage().committee().committees(cid);
        self.client.query_storage(store, at_block).await
    }

    pub async fn committees_iter(
        &self,
        page_size: u32,
        at_block: Option<Hash>,
    ) -> Result<Vec<CommitteeP<AccountId20, u32>>, subxt::Error> {
        let store = crate::node::storage().committee().committees_root();
        self.client
            .query_storage_value_iter(store, page_size, at_block)
            .await
            .map(|res| res.into_iter().map(|v| v.1).collect())
    }

    pub async fn snapshots(
        &self,
        device_mode: DeviceMode,
        at_block: Option<Hash>,
    ) -> Result<Vec<Vec<u8>>, subxt::Error> {
        let store = crate::node::storage().committee().snapshots(device_mode);
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or_default())
    }

    pub async fn all_snapshots(
        &self,
        at_block: Option<Hash>,
    ) -> Result<Vec<(DeviceMode, Vec<Vec<u8>>)>, subxt::Error> {
        let mut snapshots = Vec::new();
        for mode in [DeviceMode::Primary, DeviceMode::Normal] {
            snapshots.push((mode.clone(), self.snapshots(mode, at_block).await?));
        }
        Ok(snapshots)
    }

    pub async fn snapshots_index(
        &self,
        device_mode: DeviceMode,
        mission: MissionType,
        at_block: Option<Hash>,
    ) -> Result<Vec<u16>, subxt::Error> {
        let store = crate::node::storage()
            .committee()
            .snapshots_index(device_mode, mission);
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or_default())
    }

    pub async fn all_snapshots_index(
        &self,
        at_block: Option<Hash>,
    ) -> Result<Vec<((DeviceMode, MissionType), Vec<u16>)>, subxt::Error> {
        let mut snapshots_indexes = Vec::new();
        for mode in [DeviceMode::Primary, DeviceMode::Normal] {
            for mission in [MissionType::KeyGen, MissionType::ReShare] {
                let indexes = self
                    .snapshots_index(mode.clone(), mission.clone(), at_block)
                    .await?;
                if !indexes.is_empty() {
                    snapshots_indexes.push(((mode.clone(), mission), indexes));
                }
            }
        }
        Ok(snapshots_indexes)
    }

    pub async fn candidates_pool(
        &self,
        device_mode: DeviceMode,
        at_block: Option<Hash>,
    ) -> Result<Vec<Vec<u8>>, subxt::Error> {
        let store = crate::node::storage()
            .committee()
            .candidates_pool(device_mode);
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or_default())
    }

    pub async fn all_candidates_pool(
        &self,
        at_block: Option<Hash>,
    ) -> Result<Vec<(DeviceMode, Vec<Vec<u8>>)>, subxt::Error> {
        let mut candidates = Vec::new();
        for mode in [DeviceMode::Primary, DeviceMode::Normal] {
            candidates.push((mode.clone(), self.candidates_pool(mode, at_block).await?));
        }
        Ok(candidates)
    }

    pub async fn candidates_index(
        &self,
        device_mode: DeviceMode,
        mission: MissionType,
        at_block: Option<Hash>,
    ) -> Result<Vec<u16>, subxt::Error> {
        let store = crate::node::storage()
            .committee()
            .candidates_pool_index(device_mode, mission);
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or_default())
    }

    pub async fn all_candidates_index(
        &self,
        at_block: Option<Hash>,
    ) -> Result<Vec<((DeviceMode, MissionType), Vec<u16>)>, subxt::Error> {
        let mut candidates_indexes = Vec::new();
        for mode in [DeviceMode::Primary, DeviceMode::Normal] {
            for mission in [MissionType::KeyGen, MissionType::ReShare] {
                let indexes = self
                    .candidates_index(mode.clone(), mission.clone(), at_block)
                    .await?;
                if !indexes.is_empty() {
                    candidates_indexes.push(((mode.clone(), mission), indexes));
                }
            }
        }
        Ok(candidates_indexes)
    }

    pub async fn committee_members(
        &self,
        cid: u32,
        epoch: u32,
        fork_id: u8,
        at_block: Option<Hash>,
    ) -> Result<Option<Vec<Vec<u8>>>, subxt::Error> {
        let store = crate::node::storage()
            .committee()
            .committee_members(cid, (epoch, fork_id));
        self.client.query_storage(store, at_block).await
    }

    pub async fn member_links(
        &self,
        member: Vec<u8>,
        at_block: Option<Hash>,
    ) -> Result<u32, subxt::Error> {
        let store = crate::node::storage().committee().member_links(member);
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or_default())
    }

    pub async fn member_links_iter(
        &self,
        page_size: u32,
        at_block: Option<Hash>,
    ) -> Result<Vec<(Vec<u8>, u32)>, subxt::Error> {
        let store = crate::node::storage().committee().member_links_root();
        self.client
            .query_storage_value_iter(store, page_size, at_block)
            .await
            .map(|res| {
                res.into_iter()
                    .map(|(k, v)| (k.0[33..].to_vec(), v))
                    .collect()
            })
    }

    pub async fn candidate_links(
        &self,
        cid: u32,
        fork: u8,
        at_block: Option<Hash>,
    ) -> Result<Vec<u16>, subxt::Error> {
        let store = crate::node::storage()
            .committee()
            .candidate_links(cid, fork);
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or_default())
    }

    pub async fn epoch_change_failures(
        &self,
        cid: u32,
        fork: u8,
        at_block: Option<Hash>,
    ) -> Result<u8, subxt::Error> {
        let store = crate::node::storage()
            .committee()
            .epoch_changes_failures(cid, fork);
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or_default())
    }

    pub async fn epoch_change_failures_iter(
        &self,
        page_size: u32,
        at_block: Option<Hash>,
    ) -> Result<Vec<(u32, u8, u8)>, subxt::Error> {
        let store = crate::node::storage()
            .committee()
            .epoch_changes_failures_root();
        self.client
            .query_storage_value_iter(store, page_size, at_block)
            .await
            .map(|res| {
                res.into_iter()
                    .map(|(k, v)| {
                        let mut cid_bytes = [0u8; 4];
                        cid_bytes.copy_from_slice(&k.0[32..36]);
                        (u32::from_le_bytes(cid_bytes), k.0[36], v)
                    })
                    .collect()
            })
    }

    pub async fn committee_randomness(
        &self,
        cid: u32,
        at_block: Option<Hash>,
    ) -> Result<Option<u64>, subxt::Error> {
        let store = crate::node::storage().committee().c_randomness(cid);
        self.client.query_storage(store, at_block).await
    }

    pub async fn identity_rewards(
        &self,
        ident: Vec<u8>,
        at_block: Option<Hash>,
    ) -> Result<u128, subxt::Error> {
        let store = crate::node::storage().committee().identity_rewards(ident);
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or_default())
    }

    pub async fn exposed_identity(
        &self,
        ident: Vec<u8>,
        at_block: Option<Hash>,
    ) -> Result<Vec<u8>, subxt::Error> {
        let store = crate::node::storage().committee().exposed_identity(ident);
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or_default())
    }
}
