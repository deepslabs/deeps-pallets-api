use crate::handle_custom_error;
use subxt::utils::H256 as Hash;

pub struct CommitteeAssets<'a> {
    pub(crate) client: &'a crate::NodeClient,
}

impl<'a> CommitteeAssets<'a> {
    pub async fn update_assets(
        &self,
        cid: u32,
        block_number: u32,
        btc_asset: u128,
        brc20_assets: Vec<(Vec<u8>, u128)>,
        sender_pk: Vec<u8>,
        sender_sig: Vec<u8>,
        cmt_sig: Vec<u8>,
        fork_id: u8,
    ) -> Result<Hash, String> {
        let call = crate::node::tx().committee_assets().update_assets(
            cid,
            block_number,
            btc_asset,
            brc20_assets,
            sender_pk,
            sender_sig,
            cmt_sig,
            fork_id,
        );
        self.client
            .submit_extrinsic_without_signer(call)
            .await
            .map_err(handle_custom_error)
    }

    pub async fn update_assets_call_bytes(
        &self,
        cid: u32,
        block_number: u32,
        btc_asset: u128,
        brc20_assets: Vec<(Vec<u8>, u128)>,
        sender_pk: Vec<u8>,
        sender_sig: Vec<u8>,
        cmt_sig: Vec<u8>,
        fork_id: u8,
    ) -> Result<Vec<u8>, String> {
        let call = crate::node::tx().committee_assets().update_assets(
            cid,
            block_number,
            btc_asset,
            brc20_assets,
            sender_pk,
            sender_sig,
            cmt_sig,
            fork_id,
        );
        self.client
            .unsigned_tx_encode_to_bytes(call)
            .await
            .map_err(handle_custom_error)
    }
}
