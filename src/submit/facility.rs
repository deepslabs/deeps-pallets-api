use subxt::utils::H256 as Hash;

pub struct Facility<'a> {
    pub(crate) client: &'a crate::NodeClient,
}

impl<'a> Facility<'a> {
    pub async fn config(&self, signer: Vec<u8>, nonce: Option<u32>) -> Result<Hash, String> {
        let call = crate::node::tx().facility().config(signer);
        self.client
            .submit_extrinsic_with_signer_and_watch(call, nonce)
            .await
            .map_err(|e| e.to_string())
    }
}
