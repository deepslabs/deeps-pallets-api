use crate::node::runtime_types::fp_account::AccountId20;
use crate::node::runtime_types::pallet_channel::types::{
    BindingType, BtcCmtType, BtcScriptPair, BtcTxTunnel, Channel as ChannelP, CommitteeFeeConfig,
    ForcedWithdrawalRecord, GroupedTxInfo, MergeUtxoRecord, RefreshRecord, SlaveMessage,
    SourceTXInfo, TaprootPair, TxMessage, UidRecord, XudtInfo, XudtIssueRecord,
};
use sp_core::H256 as Hash;

pub struct Channel<'a> {
    pub(crate) client: &'a crate::NodeClient,
}

impl<'a> Channel<'a> {
    pub async fn tx_messages(
        &self,
        cid: u32,
        hash: Hash,
        at_block: Option<Hash>,
    ) -> Result<Option<TxMessage<u32>>, subxt::Error> {
        let store = crate::node::storage().channel().tx_messages(cid, hash);
        self.client.query_storage(store, at_block).await
    }

    pub async fn channel_info(
        &self,
        channel_id: u32,
        at_block: Option<Hash>,
    ) -> Result<Option<ChannelP<AccountId20>>, subxt::Error> {
        let store = crate::node::storage().channel().channel_info(channel_id);
        self.client.query_storage(store, at_block).await
    }

    pub async fn hashes_for_cid(
        &self,
        cid: u32,
        at_block: Option<Hash>,
    ) -> Result<Option<(Vec<SourceTXInfo>, BtcTxTunnel)>, subxt::Error> {
        let store = crate::node::storage().channel().hashes_for_cid(cid);
        self.client.query_storage(store, at_block).await
    }

    pub async fn source_tx_package(
        &self,
        cid: u32,
        package_key: Vec<u8>,
        at_block: Option<Hash>,
    ) -> Result<Option<Vec<SourceTXInfo>>, subxt::Error> {
        let store = crate::node::storage()
            .channel()
            .source_tx_package(cid, package_key.clone());
        self.client.query_storage(store, at_block).await
    }

    pub async fn source_hash_to_package_key(
        &self,
        chain_id: u32,
        src_hash: Vec<u8>,
        at_block: Option<Hash>,
    ) -> Result<Option<Vec<u8>>, subxt::Error> {
        let store = crate::node::storage()
            .channel()
            .source_hash_to_package_key(chain_id, src_hash.clone());
        self.client.query_storage(store, at_block).await
    }

    pub async fn btc_committee_type(
        &self,
        cid: u32,
        at_block: Option<Hash>,
    ) -> Result<Option<BtcCmtType>, subxt::Error> {
        let store = crate::node::storage().channel().btc_committee_type(cid);
        self.client.query_storage(store, at_block).await
    }

    pub async fn btc_committee_type_iter(
        &self,
        page_size: u32,
        at_block: Option<Hash>,
    ) -> Result<Vec<BtcCmtType>, subxt::Error> {
        let store = crate::node::storage().channel().btc_committee_type_root();
        self.client
            .query_storage_value_iter(store, page_size, at_block)
            .await
            .map(|res| res.into_iter().map(|v| v.1).collect())
    }

    pub async fn escape_taproot(
        &self,
        cid: u32,
        at_block: Option<Hash>,
    ) -> Result<Option<TaprootPair>, subxt::Error> {
        let store = crate::node::storage().channel().escape_taproots(cid);
        self.client.query_storage(store, at_block).await
    }

    pub async fn escape_taproot_iter(
        &self,
        page_size: u32,
        at_block: Option<Hash>,
    ) -> Result<Vec<TaprootPair>, subxt::Error> {
        let store = crate::node::storage().channel().escape_taproots_root();
        self.client
            .query_storage_value_iter(store, page_size, at_block)
            .await
            .map(|res| res.into_iter().map(|v| v.1).collect())
    }

    pub async fn bound_script(
        &self,
        cid: u32,
        at_block: Option<Hash>,
    ) -> Result<Option<BtcScriptPair>, subxt::Error> {
        let store = crate::node::storage().channel().bound_scripts(cid);
        self.client.query_storage(store, at_block).await
    }

    pub async fn bound_script_iter(
        &self,
        page_size: u32,
        at_block: Option<Hash>,
    ) -> Result<Vec<BtcScriptPair>, subxt::Error> {
        let store = crate::node::storage().channel().bound_scripts_root();
        self.client
            .query_storage_value_iter(store, page_size, at_block)
            .await
            .map(|res| res.into_iter().map(|v| v.1).collect())
    }

    pub async fn refresh_record(
        &self,
        inscription_hash: Vec<u8>,
        inscription_pos: u8,
        at_block: Option<Hash>,
    ) -> Result<Option<RefreshRecord>, subxt::Error> {
        let store = crate::node::storage()
            .channel()
            .refresh_data(inscription_hash.clone(), inscription_pos);
        self.client.query_storage(store, at_block).await
    }

    pub async fn merge_record(
        &self,
        cid: u32,
        record_hash: Vec<u8>,
        at_block: Option<Hash>,
    ) -> Result<Option<MergeUtxoRecord>, subxt::Error> {
        let store = crate::node::storage()
            .channel()
            .merge_record(cid, record_hash.clone());
        self.client.query_storage(store, at_block).await
    }

    pub async fn committee_xudt_list(
        &self,
        cid: u32,
        at_block: Option<Hash>,
    ) -> Result<Option<Vec<XudtInfo>>, subxt::Error> {
        let store = crate::node::storage().channel().committee_xudt_list(cid);
        self.client.query_storage(store, at_block).await
    }

    pub async fn committee_xudt_record(
        &self,
        cid: u32,
        args_of_token: Vec<u8>,
        at_block: Option<Hash>,
    ) -> Result<Option<XudtIssueRecord>, subxt::Error> {
        let store = crate::node::storage()
            .channel()
            .committee_xudt_record(cid, args_of_token.clone());
        self.client.query_storage(store, at_block).await
    }

    pub async fn uid_consensus_record(
        &self,
        cid: u32,
        uid: Vec<u8>,
        at_block: Option<Hash>,
    ) -> Result<Option<UidRecord<u32>>, subxt::Error> {
        let store = crate::node::storage()
            .channel()
            .uid_consensus_record(cid, uid.clone());
        self.client.query_storage(store, at_block).await
    }

    pub async fn committee_fee_data(
        &self,
        cid: u32,
        at_block: Option<Hash>,
    ) -> Result<Option<CommitteeFeeConfig>, subxt::Error> {
        let store = crate::node::storage().channel().committee_fee_data(cid);
        self.client.query_storage(store, at_block).await
    }

    pub async fn committee_fee_data_iter(
        &self,
        page_size: u32,
        at_block: Option<Hash>,
    ) -> Result<Vec<(u32, CommitteeFeeConfig)>, subxt::Error> {
        let store = crate::node::storage().channel().committee_fee_data_root();
        self.client
            .query_storage_value_iter(store, page_size, at_block)
            .await
            .map(|res| {
                res.into_iter()
                    .map(|(key, value)| {
                        let mut cid_bytes = [0u8; 4];
                        cid_bytes.copy_from_slice(&key.0[48..]);
                        (u32::from_le_bytes(cid_bytes), value)
                    })
                    .collect()
            })
    }

    pub async fn channel_mapping_tick_iter(
        &self,
        page_size: u32,
        at_block: Option<Hash>,
    ) -> Result<Vec<(u32, Vec<(Vec<u8>, Vec<u8>)>)>, subxt::Error> {
        let store = crate::node::storage().channel().channel_mapping_tick_root();
        self.client
            .query_storage_value_iter(store, page_size, at_block)
            .await
            .map(|res| {
                res.into_iter()
                    .map(|(key, value)| {
                        let mut cid_bytes = [0u8; 4];
                        cid_bytes.copy_from_slice(&key.0[48..]);
                        (u32::from_le_bytes(cid_bytes), value)
                    })
                    .collect()
            })
    }

    pub async fn channel_mapping_tick(
        &self,
        channel_id: u32,
        at_block: Option<Hash>,
    ) -> Result<Option<Vec<(Vec<u8>, Vec<u8>)>>, subxt::Error> {
        let store = crate::node::storage()
            .channel()
            .channel_mapping_tick(channel_id);
        self.client.query_storage(store, at_block).await
    }

    pub async fn forced_withdrawal_record(
        &self,
        nonce_key: u128,
        at_block: Option<Hash>,
    ) -> Result<Option<ForcedWithdrawalRecord>, subxt::Error> {
        let store = crate::node::storage()
            .channel()
            .forced_withdrawal_data(nonce_key);
        self.client.query_storage(store, at_block).await
    }

    pub async fn committee_binding_info(
        &self,
        cid: u32,
        at_block: Option<Hash>,
    ) -> Result<Option<BindingType>, subxt::Error> {
        let store = crate::node::storage().channel().committee_binding_info(cid);
        self.client.query_storage(store, at_block).await
    }

    pub async fn committee_binding_info_iter(
        &self,
        page_size: u32,
        at_block: Option<Hash>,
    ) -> Result<Vec<(u32, BindingType)>, subxt::Error> {
        let store = crate::node::storage()
            .channel()
            .committee_binding_info_root();
        self.client
            .query_storage_value_iter(store, page_size, at_block)
            .await
            .map(|res| {
                res.into_iter()
                    .map(|(key, value)| {
                        let mut cid_bytes = [0u8; 4];
                        cid_bytes.copy_from_slice(&key.0[48..]);
                        (u32::from_le_bytes(cid_bytes), value)
                    })
                    .collect()
            })
    }

    pub async fn slave_messages(
        &self,
        cid: u32,
        hash: Hash,
        at_block: Option<Hash>,
    ) -> Result<Option<SlaveMessage>, subxt::Error> {
        let store = crate::node::storage().channel().slave_messages(cid, hash);
        self.client.query_storage(store, at_block).await
    }

    pub async fn collect_slave_signatures(
        &self,
        master_cid: u32,
        hash: Hash,
    ) -> Result<Vec<Vec<u8>>, subxt::Error> {
        let binding_info = self.committee_binding_info(master_cid, None).await?;
        let mut slave_sigs = Vec::new();
        if let Some(info) = binding_info {
            match info {
                BindingType::Slave(slave_cids) => {
                    for slave_cid in slave_cids {
                        let slave_message = match self.slave_messages(slave_cid, hash, None).await?
                        {
                            Some(slave_message) => slave_message,
                            None => {
                                log::warn!(target: "event_handler", "fetch slave message failed for cid: {slave_cid}, hash: {hash:?}");
                                continue;
                            }
                        };
                        slave_sigs.push(slave_message.signature);
                    }
                }
                _ => {}
            }
        }
        Ok(slave_sigs)
    }

    pub async fn grouped_tx_infos(
        &self,
        cid: u32,
        hash: Hash,
        at_block: Option<Hash>,
    ) -> Result<Option<GroupedTxInfo>, subxt::Error> {
        let store = crate::node::storage().channel().grouped_tx_infos(cid, hash);
        self.client.query_storage(store, at_block).await
    }

    pub async fn collect_grouped_messages(
        &self,
        cid: u32,
        hash: Hash,
        is_valid: bool,
    ) -> Result<Option<Vec<Vec<u8>>>, subxt::Error> {
        let grouped_info = self.grouped_tx_infos(cid, hash, None).await?;
        if let Some(info) = grouped_info {
            let mut sub_msgs = Vec::new();
            let grouped_hashes = if is_valid {
                info.indexes
                    .iter()
                    .filter_map(|&idx| {
                        let index = idx as usize;
                        if index < info.hashes.len() {
                            Some(info.hashes[index])
                        } else {
                            None
                        }
                    })
                    .collect()
            } else {
                info.hashes
            };
            for sub_hash in grouped_hashes {
                if let Ok(Some(tx_msg)) = self.tx_messages(cid, sub_hash.into(), None).await {
                    sub_msgs.push(tx_msg.msg);
                } else {
                    log::warn!(target: "event_handler", "fetch sub tx message failed for cid: {cid}, sub_hash: {sub_hash:?}");
                }
            }
            Ok(Some(sub_msgs))
        } else {
            Ok(None)
        }
    }
}
