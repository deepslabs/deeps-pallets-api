use crate::node::runtime_types::pallet_mining::types::{DeviceMode, MonitorType, OnChainPayload};
use crate::handle_custom_error;
use subxt::utils::H256 as Hash;

pub struct Mining<'a> {
    pub(crate) client: &'a crate::NodeClient,
}

impl<'a> Mining<'a> {
    pub async fn im_online(&self, payload: OnChainPayload) -> Result<Hash, String> {
        let call = crate::node::tx().mining().im_online(payload);
        self.client
            .submit_extrinsic_without_signer(call)
            .await
            .map_err(handle_custom_error)
    }

    pub async fn report_standby(
        &self,
        id: Vec<u8>,
        version: u16,
        enclave_hash: Vec<u8>,
        signature: Vec<u8>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx()
            .mining()
            .report_standby(id, version, enclave_hash, signature);
        self.client
            .submit_extrinsic_without_signer(call)
            .await
            .map_err(handle_custom_error)
    }

    pub async fn register_device(
        &self,
        owner: crate::node::runtime_types::fp_account::AccountId20,
        report: Vec<u8>,
        version: u16,
        identity: Vec<u8>,
        device_mode: DeviceMode,
        monitor_type: MonitorType,
        signature: Vec<u8>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx().mining().register_device_with_ident(
            owner,
            report,
            version,
            identity,
            device_mode,
            monitor_type,
            signature,
        );
        let tx_process = self
            .client
            .submit_extrinsic_without_signer_and_watch(call)
            .await
            .map_err(|e| e.to_string())?;
        match tx_process.wait_for_finalized().await {
            Ok(tx) => Ok(tx
                .wait_for_success()
                .await
                .map_err(|e| e.to_string())?
                .extrinsic_hash()),
            Err(e) => Err(e.to_string()),
        }
    }

    pub async fn update_votes(
        &self,
        changed_votes: Vec<(Vec<u8>, u128)>,
        nonce: Option<u32>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx().mining().update_votes(changed_votes);
        self.client
            .submit_extrinsic_with_signer_and_watch(call, nonce)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn join_service(&self, id: Vec<u8>, nonce: Option<u32>) -> Result<Hash, String> {
        let call = crate::node::tx().mining().join_service(id);
        self.client
            .submit_extrinsic_with_signer_and_watch(call, nonce)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn exit_service(&self, id: Vec<u8>, nonce: Option<u32>) -> Result<Hash, String> {
        let call = crate::node::tx().mining().exit_service(id);
        self.client
            .submit_extrinsic_with_signer_and_watch(call, nonce)
            .await
            .map_err(|e| e.to_string())
    }
}
