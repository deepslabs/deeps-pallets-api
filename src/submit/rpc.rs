use subxt::utils::H256 as Hash;

pub struct Rpc<'a> {
    pub(crate) client: &'a crate::NodeClient,
}

impl<'a> Rpc<'a> {
    pub async fn register_device(
        &self,
        owner: crate::node::runtime_types::fp_account::AccountId20,
        report: Vec<u8>,
        version: u16,
        signature: Vec<u8>,
        deviceid: Vec<u8>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx()
            .rpc()
            .register_device(owner, report, version, signature, deviceid);
        self.client
            .submit_extrinsic_without_signer(call)
            .await
            .map_err(|e| e.to_string())
    }
}
