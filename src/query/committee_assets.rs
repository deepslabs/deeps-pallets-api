use crate::node::runtime_types::pallet_committee_assets::pallet::AssetConsensusInfo;
use subxt::utils::H256 as Hash;

pub struct CommitteeAssets<'a> {
    pub(crate) client: &'a crate::NodeClient,
}

impl<'a> CommitteeAssets<'a> {
    pub async fn all_concerned_brc20(
        &self,
        at_block: Option<Hash>,
    ) -> Result<Option<Vec<Vec<u8>>>, subxt::Error> {
        let store = crate::node::storage()
            .committee_assets()
            .all_concerned_brc20();
        self.client.query_storage(store, at_block).await
    }

    pub async fn brc20_decimals(
        &self,
        tick: Vec<u8>,
        at_block: Option<Hash>,
    ) -> Result<Option<u8>, subxt::Error> {
        let store = crate::node::storage()
            .committee_assets()
            .brc20_decimals(tick);
        self.client.query_storage(store, at_block).await
    }

    pub async fn assets_consensus(
        &self,
        cid: u32,
        at_block: Option<Hash>,
    ) -> Result<Option<AssetConsensusInfo>, subxt::Error> {
        let store = crate::node::storage()
            .committee_assets()
            .assets_consensus(cid);
        self.client.query_storage(store, at_block).await
    }
}
