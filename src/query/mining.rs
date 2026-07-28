use crate::node::runtime_types::{
    fp_account::AccountId20,
    pallet_facility::pallet::DIdentity,
    pallet_mining::types::{DeviceInfo, DeviceMode, MonitorState, RegisterData},
    primitive_types::U256,
    sp_arithmetic::per_things::Perbill,
};
use subxt::utils::H256 as Hash;

pub struct Mining<'a> {
    pub(crate) client: &'a crate::NodeClient,
}

impl<'a> Mining<'a> {
    pub async fn challenges(
        &self,
        session: u32,
        at_block: Option<Hash>,
    ) -> Result<Option<U256>, subxt::Error> {
        let store = crate::node::storage().mining().challenges(session);
        self.client.query_storage(store, at_block).await
    }

    pub async fn working_devices(
        &self,
        session: Option<u32>,
        at_block: Option<Hash>,
    ) -> Result<Option<(Vec<(DIdentity, bool)>, u32)>, subxt::Error> {
        let session = match session {
            Some(session) => session,
            None => {
                let client = self.client.client.read().await.blocks();
                let current_block = match at_block {
                    Some(hash) => client.at(hash).await,
                    None => client.at_latest().await,
                };
                let current_number = current_block.map(|b| b.number())?;
                let constant_query = crate::node::constants().mining().era_block_number();
                self.client
                    .query_constant(constant_query)
                    .await
                    .map(|era_block_number| current_number / era_block_number)?
            }
        };
        let store = crate::node::storage().mining().working_devices(session);
        self.client
            .query_storage(store, at_block)
            .await
            .map(|res| res.and_then(|data| Some((data, session))))
    }

    pub async fn device_info(
        &self,
        id: Vec<u8>,
        at_block: Option<Hash>,
    ) -> Result<Option<DeviceInfo<AccountId20, u32, u128>>, subxt::Error> {
        let storage_query = crate::node::storage().mining().devices(id.clone());
        self.client.query_storage(storage_query, at_block).await
    }

    pub async fn device_info_iter(
        &self,
        at_block: Option<Hash>,
    ) -> Result<Vec<DeviceInfo<AccountId20, u32, u128>>, subxt::Error> {
        let storage_query = crate::node::storage().mining().devices_root();
        self.client
            .query_storage_value_iter(storage_query, 300, at_block)
            .await
            .map(|res| res.into_iter().map(|v| v.1).collect())
    }

    pub async fn device_identity_map(
        &self,
        id: Vec<u8>,
        at_block: Option<Hash>,
    ) -> Result<Option<Vec<u8>>, subxt::Error> {
        let storage_query = crate::node::storage().mining().device_identity_map(id);
        self.client.query_storage(storage_query, at_block).await
    }

    pub async fn device_identity_map_iter(
        &self,
        page_size: u32,
        at_block: Option<Hash>,
    ) -> Result<Vec<(Vec<u8>, Vec<u8>)>, subxt::Error> {
        let storage_query = crate::node::storage().mining().device_identity_map_root();
        self.client
            .query_storage_value_iter(storage_query, page_size, at_block)
            .await
            .map(|res| {
                res.into_iter()
                    .map(|(k, v)| (k.0[49..].to_vec(), v))
                    .collect()
            })
    }

    pub async fn device_monitor_state(
        &self,
        id: Vec<u8>,
        at_block: Option<Hash>,
    ) -> Result<Option<MonitorState>, subxt::Error> {
        let storage_query = crate::node::storage().mining().device_monitor_state(id);
        self.client.query_storage(storage_query, at_block).await
    }

    pub async fn device_stake_for_current_epoch(
        &self,
        id: Vec<u8>,
        at_block: Option<Hash>,
    ) -> Result<Option<u128>, subxt::Error> {
        let storage_query = crate::node::storage()
            .mining()
            .device_stake_for_current_epoch(id.clone());
        self.client.query_storage(storage_query, at_block).await
    }

    pub async fn device_votes_for_current_epoch(
        &self,
        id: Vec<u8>,
        at_block: Option<Hash>,
    ) -> Result<Option<Vec<(AccountId20, u128)>>, subxt::Error> {
        let storage_query = crate::node::storage()
            .mining()
            .device_votes_for_current_epoch(id.clone());
        self.client.query_storage(storage_query, at_block).await
    }

    pub async fn device_votes_for_next_epoch(
        &self,
        id: Vec<u8>,
        at_block: Option<Hash>,
    ) -> Result<Option<Vec<(AccountId20, u128)>>, subxt::Error> {
        let storage_query = crate::node::storage()
            .mining()
            .device_votes_for_next_epoch(id.clone());
        self.client.query_storage(storage_query, at_block).await
    }

    pub async fn device_data(
        &self,
        did: DIdentity,
        at_block: Option<Hash>,
    ) -> Result<Option<Vec<u8>>, subxt::Error> {
        let store = crate::node::storage().mining().device_data(did.clone());
        self.client.query_storage(store, at_block).await
    }

    pub async fn devices_iter(
        &self,
        page_size: u32,
        at_block: Option<Hash>,
    ) -> Result<Vec<DeviceInfo<AccountId20, u32, u128>>, subxt::Error> {
        let store = crate::node::storage().mining().devices_root();
        self.client
            .query_storage_value_iter(store, page_size, at_block)
            .await
            .map(|res| res.into_iter().map(|(_, v)| v).collect())
    }

    pub async fn device_register_data(
        &self,
        device_id: Vec<u8>,
        at_block: Option<Hash>,
    ) -> Result<Option<RegisterData>, subxt::Error> {
        let store = crate::node::storage()
            .mining()
            .device_register_data(device_id);
        self.client.query_storage(store, at_block).await
    }

    pub async fn device_register_data_iter(
        &self,
        page_size: u32,
        at_block: Option<Hash>,
    ) -> Result<Vec<(Vec<u8>, RegisterData)>, subxt::Error> {
        let store = crate::node::storage().mining().device_register_data_root();
        self.client
            .query_storage_value_iter(store, page_size, at_block)
            .await
            .map(|res| {
                res.into_iter()
                    .map(|(k, v)| (k.0[49..].to_vec(), v))
                    .collect()
            })
    }

    pub async fn foundation(
        &self,
        at_block: Option<Hash>,
    ) -> Result<Option<AccountId20>, subxt::Error> {
        let store = crate::node::storage().mining().foundation();
        self.client.query_storage(store, at_block).await
    }

    pub async fn foundation_reward_rate(
        &self,
        at_block: Option<Hash>,
    ) -> Result<Perbill, subxt::Error> {
        let store = crate::node::storage().mining().foundation_reward_rate();
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or(Perbill(750_000_000)))
    }

    pub async fn base_reward_rate(&self, at_block: Option<Hash>) -> Result<Perbill, subxt::Error> {
        let store = crate::node::storage().mining().base_reward_rate();
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or(Perbill(100_000_000)))
    }

    pub async fn rewards_for_epoch(
        &self,
        epoch: u64,
        at_block: Option<Hash>,
    ) -> Result<u128, subxt::Error> {
        let store = crate::node::storage().mining().rewards_for_epoch(epoch);
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or_default())
    }

    pub async fn incentive_rewards_for_epoch(
        &self,
        epoch: u64,
        at_block: Option<Hash>,
    ) -> Result<u128, subxt::Error> {
        let store = crate::node::storage()
            .mining()
            .incentive_rewards_for_epoch(epoch);
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or_default())
    }

    pub async fn number_of_pay_rewards_in_one_block(
        &self,
        at_block: Option<Hash>,
    ) -> Result<u64, subxt::Error> {
        let store = crate::node::storage()
            .mining()
            .number_of_pay_rewards_in_one_block();
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or_default())
    }

    pub async fn device_stake_map(
        &self,
        device_id: Vec<u8>,
        at_block: Option<Hash>,
    ) -> Result<Vec<u8>, subxt::Error> {
        let store = crate::node::storage()
            .mining()
            .device_stake_map(device_id.clone());
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or(device_id))
    }

    pub async fn stake_device_map(
        &self,
        stake_id: Vec<u8>,
        at_block: Option<Hash>,
    ) -> Result<Vec<u8>, subxt::Error> {
        let store = crate::node::storage()
            .mining()
            .stake_device_map(stake_id.clone());
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or(stake_id))
    }

    pub async fn rewards_from_committee(
        &self,
        device_id: Vec<u8>,
        epoch: u64,
        at_block: Option<Hash>,
    ) -> Result<(u128, u128), subxt::Error> {
        let store = crate::node::storage()
            .mining()
            .rewards_from_committee(device_id, epoch);
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or((0, 0)))
    }

    pub async fn total_score_for_epoch(
        &self,
        device_mode: DeviceMode,
        epoch: u64,
        at_block: Option<Hash>,
    ) -> Result<u128, subxt::Error> {
        let store = crate::node::storage().mining().total_score_for_epoch_by_mode(device_mode, epoch);
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or_default())
    }

    pub async fn scores_for_epoch(
        &self,
        device_id: Vec<u8>,
        epoch: u64,
        at_block: Option<Hash>,
    ) -> Result<u128, subxt::Error> {
        let store = crate::node::storage()
            .mining()
            .scores_for_epoch(device_id, epoch);
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or_default())
    }

    pub async fn total_committee_score_for_epoch(
        &self,
        device_mode: DeviceMode,
        epoch: u64,
        at_block: Option<Hash>,
    ) -> Result<u128, subxt::Error> {
        let store = crate::node::storage()
            .mining()
            .total_committee_score_for_epoch_by_mode(device_mode, epoch);
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or_default())
    }

    pub async fn committee_scores_for_epoch(
        &self,
        epoch: u64,
        at_block: Option<Hash>,
    ) -> Result<Option<Vec<(Vec<u8>, u32, u128)>>, subxt::Error> {
        let store = crate::node::storage()
            .mining()
            .committee_scores_for_epoch(epoch);
        self.client.query_storage(store, at_block).await
    }

    pub async fn device_ids_waiting_pay_rewards_for_epoch(
        &self,
        epoch: u64,
        at_block: Option<Hash>,
    ) -> Result<Vec<Vec<u8>>, subxt::Error> {
        let store = crate::node::storage()
            .mining()
            .device_ids_waiting_pay_rewards_for_epoch(epoch);
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or_default())
    }

    pub async fn device_commission_for_current_epoch(
        &self,
        device_id: Vec<u8>,
        at_block: Option<Hash>,
    ) -> Result<Perbill, subxt::Error> {
        let store = crate::node::storage()
            .mining()
            .device_commission_for_current_epoch(device_id);
        self.client
            .query_storage(store, at_block)
            .await
            .map(|r| r.unwrap_or(Perbill(150_000_000)))
    }
}
