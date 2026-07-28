use crate::node::runtime_types::pallet_mining::types::DeviceMode;
use crate::handle_custom_error;
use subxt::utils::H256 as Hash;

pub struct CommitteeHealth<'a> {
    pub(crate) client: &'a crate::NodeClient,
}

impl<'a> CommitteeHealth<'a> {
    pub async fn report_health(&self, ident: Vec<u8>, sig: Vec<u8>) -> Result<Hash, String> {
        let call = crate::node::tx()
            .committee_health()
            .report_health(ident, sig);
        self.client
            .submit_extrinsic_without_signer(call)
            .await
            .map_err(|e| handle_custom_error(e))
    }

    pub async fn report_health_call_bytes(
        &self,
        ident: Vec<u8>,
        sig: Vec<u8>,
    ) -> Result<Vec<u8>, String> {
        let call = crate::node::tx()
            .committee_health()
            .report_health(ident, sig);
        self.client
            .unsigned_tx_encode_to_bytes(call)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn report_state_vote(
        &self,
        device_id: Vec<u8>,
        sig: Vec<u8>,
        device_mode: DeviceMode,
    ) -> Result<Hash, String> {
        let call =
            crate::node::tx()
                .committee_health()
                .report_state_vote(device_id, sig, device_mode);
        self.client
            .submit_extrinsic_without_signer(call)
            .await
            .map_err(|e| handle_custom_error(e))
    }

    pub async fn report_state_vote_call_bytes(
        &self,
        device_id: Vec<u8>,
        sig: Vec<u8>,
        device_mode: DeviceMode,
    ) -> Result<Vec<u8>, String> {
        let call =
            crate::node::tx()
                .committee_health()
                .report_state_vote(device_id, sig, device_mode);
        self.client
            .unsigned_tx_encode_to_bytes(call)
            .await
            .map_err(|e| e.to_string())
    }
}
