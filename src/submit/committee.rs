#![allow(clippy::too_many_arguments)]
use crate::node::runtime_types::pallet_committee::types::{CommitteeMode, CryptoType, MissionType};
use crate::node::runtime_types::pallet_mining::types::DeviceMode;
use crate::handle_custom_error;
use subxt::utils::H256 as Hash;

pub struct Committee<'a> {
    pub(crate) client: &'a crate::NodeClient,
}

impl<'a> Committee<'a> {
    pub async fn create_committee(
        &self,
        t: u16,
        n: u16,
        crypto: CryptoType,
        fork: u8,
        mode: CommitteeMode,
        nonce: Option<u32>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx()
            .committee()
            .create_committee(t, n, crypto, fork, mode);
        self.client
            .submit_extrinsic_with_signer_and_watch(call, nonce)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn enter_epoch(
        &self,
        device_mode: DeviceMode,
        epoch: u64,
        proofs: Vec<(MissionType, Vec<u8>, Vec<u8>, Vec<u8>)>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx()
            .committee()
            .enter_epoch(device_mode, epoch, proofs);
        self.client
            .submit_extrinsic_without_signer(call)
            .await
            .map_err(handle_custom_error)
    }

    pub async fn enter_epoch_call_bytes(
        &self,
        device_mode: DeviceMode,
        epoch: u64,
        proofs: Vec<(MissionType, Vec<u8>, Vec<u8>, Vec<u8>)>,
    ) -> Result<Vec<u8>, String> {
        let call = crate::node::tx()
            .committee()
            .enter_epoch(device_mode, epoch, proofs);
        self.client
            .unsigned_tx_encode_to_bytes(call)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn expose_identity(
        &self,
        identity: Vec<u8>,
        joins: Vec<(u32, Vec<(u8, u32, u32)>)>,
        device_id: Vec<u8>,
        ident_sig: Vec<u8>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx()
            .committee()
            .expose_identity(identity, joins, device_id, ident_sig);
        self.client
            .submit_extrinsic_without_signer(call)
            .await
            .map_err(handle_custom_error)
    }

    pub async fn expose_identity_call_bytes(
        &self,
        identity: Vec<u8>,
        joins: Vec<(u32, Vec<(u8, u32, u32)>)>,
        device_id: Vec<u8>,
        ident_sig: Vec<u8>,
    ) -> Result<Vec<u8>, String> {
        let call = crate::node::tx()
            .committee()
            .expose_identity(identity, joins, device_id, ident_sig);
        self.client
            .unsigned_tx_encode_to_bytes(call)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn active_committee(
        &self,
        cid: u32,
        chain_id: u32,
        address: Vec<u8>,
        nonce: Option<u32>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx()
            .committee()
            .active_committee(cid, chain_id, address);
        self.client
            .submit_extrinsic_with_signer_and_watch(call, nonce)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn report_change(
        &self,
        pk: Vec<u8>,
        sig: Vec<u8>,
        cid: u32,
        epoch: u32,
        fork_id: u8,
        signature: Vec<u8>,
        pubkey: Vec<u8>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx()
            .committee()
            .report_change(pk, sig, cid, epoch, fork_id, signature, pubkey);
        self.client
            .submit_extrinsic_without_signer(call)
            .await
            .map_err(handle_custom_error)
    }

    pub async fn report_change_call_bytes(
        &self,
        pk: Vec<u8>,
        sig: Vec<u8>,
        cid: u32,
        epoch: u32,
        fork_id: u8,
        signature: Vec<u8>,
        pubkey: Vec<u8>,
    ) -> Result<Vec<u8>, String> {
        let call = crate::node::tx()
            .committee()
            .report_change(pk, sig, cid, epoch, fork_id, signature, pubkey);
        self.client
            .unsigned_tx_encode_to_bytes(call)
            .await
            .map_err(|e| e.to_string())
    }
}
