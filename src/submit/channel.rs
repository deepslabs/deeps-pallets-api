use crate::handle_custom_error;
use crate::node::runtime_types::pallet_channel::types::{
    CmtType, HandleConnection, TaprootType, TxSource, XudtStatus,
};
use sp_core::H256 as Hash;

pub struct Channel<'a> {
    pub(crate) client: &'a crate::NodeClient,
}

impl<'a> Channel<'a> {
    pub async fn create_channel(
        &self,
        info: Vec<u8>,
        connections: Vec<HandleConnection>,
        nonce: Option<u32>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx()
            .channel()
            .create_channel(info, connections);
        self.client
            .submit_extrinsic_with_signer_and_watch(call, nonce)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn bind_committees(
        &self,
        channel_id: u32,
        connections: Vec<HandleConnection>,
        nonce: Option<u32>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx()
            .channel()
            .bind_committees(channel_id, connections);
        self.client
            .submit_extrinsic_with_signer_and_watch(call, nonce)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn submit_transaction(
        &self,
        channel_id: u32,
        cid: u32,
        msg: Vec<u8>,
        source: TxSource,
        need_watch_res: bool,
        nonce: Option<u32>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx()
            .channel()
            .import_new_tx(channel_id, cid, msg, source);
        if need_watch_res {
            self.client
                .submit_extrinsic_with_signer_and_watch(call, nonce)
                .await
                .map_err(|e| e.to_string())
        } else {
            self.client
                .submit_extrinsic_with_signer_without_watch(call, nonce)
                .await
                .map_err(|e| e.to_string())
        }
    }

    pub async fn import_new_src_hash(
        &self,
        cid: u32,
        hash: Vec<u8>,
        src_chain_id: u32,
        uid: Vec<u8>,
        need_watch_res: bool,
        nonce: Option<u32>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx()
            .channel()
            .import_new_source_hash(cid, hash, src_chain_id, uid);
        if need_watch_res {
            self.client
                .submit_extrinsic_with_signer_and_watch(call, nonce)
                .await
                .map_err(|e| e.to_string())
        } else {
            self.client
                .submit_extrinsic_with_signer_without_watch(call, nonce)
                .await
                .map_err(|e| e.to_string())
        }
    }

    pub async fn report_result(
        &self,
        pk: Vec<u8>,
        sig: Vec<u8>,
        cid: u32,
        fork_id: u8,
        hash: Hash,
        signature: Vec<u8>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx()
            .channel()
            .submit_tx_sign_result(pk, sig, cid, fork_id, hash, signature);
        self.client
            .submit_extrinsic_without_signer(call)
            .await
            .map_err(handle_custom_error)
    }

    pub async fn report_result_call_bytes(
        &self,
        pk: Vec<u8>,
        sig: Vec<u8>,
        cid: u32,
        fork_id: u8,
        hash: Hash,
        signature: Vec<u8>,
        valid_grouped_index: Option<Vec<u16>>,
    ) -> Result<Vec<u8>, String> {
        if let Some(grouped_index) = valid_grouped_index {
            let call = crate::node::tx().channel().submit_grouped_tx_sign_result(
                pk,
                sig,
                cid,
                fork_id,
                hash,
                signature,
                grouped_index,
            );
            self.client
                .unsigned_tx_encode_to_bytes(call)
                .await
                .map_err(|e| e.to_string())
        } else {
            let call = crate::node::tx()
                .channel()
                .submit_tx_sign_result(pk, sig, cid, fork_id, hash, signature);
            self.client
                .unsigned_tx_encode_to_bytes(call)
                .await
                .map_err(|e| e.to_string())
        }
    }

    pub async fn request_sign(
        &self,
        cid: u32,
        hash: Hash,
        nonce: Option<u32>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx().channel().request_sign(cid, hash);
        self.client
            .submit_extrinsic_with_signer_and_watch(call, nonce)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn sync_status(
        &self,
        cid: u32,
        hash: Vec<u8>,
        watch_res: bool,
        nonce: Option<u32>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx().channel().sync_status(cid, hash);
        if watch_res {
            self.client
                .submit_extrinsic_with_signer_and_watch(call, nonce)
                .await
                .map_err(|e| e.to_string())
        } else {
            self.client
                .submit_extrinsic_with_signer_without_watch(call, nonce)
                .await
                .map_err(|e| e.to_string())
        }
    }

    pub async fn clear_target_package(
        &self,
        cid: u32,
        package_key: Vec<u8>,
        watch_res: bool,
        nonce: Option<u32>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx()
            .channel()
            .clear_target_package(cid, package_key);
        if watch_res {
            self.client
                .submit_extrinsic_with_signer_and_watch(call, nonce)
                .await
                .map_err(|e| e.to_string())
        } else {
            self.client
                .submit_extrinsic_with_signer_without_watch(call, nonce)
                .await
                .map_err(|e| e.to_string())
        }
    }

    pub async fn create_channel_with_taproot(
        &self,
        info: Vec<u8>,
        connections: Vec<((u32, Option<u32>), u32, Vec<u8>, CmtType)>,
        taproot_types: Vec<(u32, TaprootType)>,
        nonce: Option<u32>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx().channel().create_channel_with_taproot(
            info,
            connections,
            taproot_types,
        );
        self.client
            .submit_extrinsic_with_signer_and_watch(call, nonce)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn request_to_sign_refresh(
        &self,
        cid: u32,
        inscription_tx: Vec<u8>,
        inscription_pos: u8,
        msg: Vec<u8>,
        watch_res: bool,
        nonce: Option<u32>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx().channel().request_to_sign_refresh(
            cid,
            inscription_tx,
            inscription_pos,
            msg,
        );
        if watch_res {
            self.client
                .submit_extrinsic_with_signer_and_watch(call, nonce)
                .await
                .map_err(|e| e.to_string())
        } else {
            self.client
                .submit_extrinsic_with_signer_without_watch(call, nonce)
                .await
                .map_err(|e| e.to_string())
        }
    }

    pub async fn request_to_sign_merge_tx(
        &self,
        cid: u32,
        record_hash: Vec<u8>,
        msg: Vec<u8>,
        watch_res: bool,
        nonce: Option<u32>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx()
            .channel()
            .request_to_sign_merge_tx(cid, record_hash, msg);
        if watch_res {
            self.client
                .submit_extrinsic_with_signer_and_watch(call, nonce)
                .await
                .map_err(|e| e.to_string())
        } else {
            self.client
                .submit_extrinsic_with_signer_without_watch(call, nonce)
                .await
                .map_err(|e| e.to_string())
        }
    }

    pub async fn submit_refresh_result(
        &self,
        cid: u32,
        inscription_tx: Vec<u8>,
        inscription_pos: u8,
        sender_pk: Vec<u8>,
        sender_sig: Vec<u8>,
        cmt_sig: Vec<u8>,
        fork_id: u8,
    ) -> Result<Hash, String> {
        let call = crate::node::tx().channel().submit_refresh_result(
            cid,
            inscription_tx,
            inscription_pos,
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

    pub async fn submit_refresh_result_call_bytes(
        &self,
        cid: u32,
        inscription_tx: Vec<u8>,
        inscription_pos: u8,
        sender_pk: Vec<u8>,
        sender_sig: Vec<u8>,
        cmt_sig: Vec<u8>,
        fork_id: u8,
    ) -> Result<Vec<u8>, String> {
        let call = crate::node::tx().channel().submit_refresh_result(
            cid,
            inscription_tx,
            inscription_pos,
            sender_pk,
            sender_sig,
            cmt_sig,
            fork_id,
        );
        self.client
            .unsigned_tx_encode_to_bytes(call)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn submit_merge_tx_result(
        &self,
        cid: u32,
        record_hash: Vec<u8>,
        sender_pk: Vec<u8>,
        sender_sig: Vec<u8>,
        cmt_sig: Vec<u8>,
        fork_id: u8,
    ) -> Result<Hash, String> {
        let call = crate::node::tx().channel().submit_merge_tx_result(
            cid,
            record_hash,
            sender_pk,
            sender_sig,
            cmt_sig,
            fork_id,
        );
        self.client
            .submit_extrinsic_without_signer(call)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn submit_merge_tx_result_call_bytes(
        &self,
        cid: u32,
        record_hash: Vec<u8>,
        sender_pk: Vec<u8>,
        sender_sig: Vec<u8>,
        cmt_sig: Vec<u8>,
        fork_id: u8,
    ) -> Result<Vec<u8>, String> {
        let call = crate::node::tx().channel().submit_merge_tx_result(
            cid,
            record_hash,
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

    pub async fn sign_issue_xudt(
        &self,
        cid: u32,
        args_of_token: Vec<u8>,
        msg: Vec<u8>,
        nonce: Option<u32>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx()
            .channel()
            .sign_issue_xudt(cid, args_of_token, msg);
        self.client
            .submit_extrinsic_with_signer_and_watch(call, nonce)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn submit_issue_xudt_sign_result(
        &self,
        cid: u32,
        args_of_token: Vec<u8>,
        pk: Vec<u8>,
        sig: Vec<u8>,
        fork_id: u8,
        signature: Vec<u8>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx().channel().submit_issue_xudt_sign_result(
            cid,
            args_of_token,
            pk,
            sig,
            fork_id,
            signature,
        );
        self.client
            .submit_extrinsic_without_signer(call)
            .await
            .map_err(handle_custom_error)
    }

    pub async fn submit_issue_xudt_sign_result_call_bytes(
        &self,
        cid: u32,
        args_of_token: Vec<u8>,
        pk: Vec<u8>,
        sig: Vec<u8>,
        fork_id: u8,
        signature: Vec<u8>,
    ) -> Result<Vec<u8>, String> {
        let call = crate::node::tx().channel().submit_issue_xudt_sign_result(
            cid,
            args_of_token,
            pk,
            sig,
            fork_id,
            signature,
        );
        self.client
            .unsigned_tx_encode_to_bytes(call)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn sync_issue_xudt_result(
        &self,
        cid: u32,
        args_of_token: Vec<u8>,
        status: XudtStatus,
        nonce: Option<u32>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx()
            .channel()
            .sync_issue_xudt_result(cid, args_of_token, status);
        self.client
            .submit_extrinsic_with_signer_and_watch(call, nonce)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn update_src_hash_seq(
        &self,
        cid: u32, // dst_cid
        src_chain: u32,
        src_hash: Vec<u8>,
        nonce: Option<u32>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx()
            .channel()
            .update_src_hash_seq(cid, src_chain, src_hash);
        self.client
            .submit_extrinsic_with_signer_without_watch(call, nonce)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn submit_uid_sign_result(
        &self,
        cid: u32,
        uid: Vec<u8>,
        pk: Vec<u8>,
        sig: Vec<u8>,
        fork_id: u8,
        signature: Vec<u8>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx()
            .channel()
            .submit_uid_sign_result(cid, uid, pk, sig, fork_id, signature);
        self.client
            .submit_extrinsic_without_signer(call)
            .await
            .map_err(handle_custom_error)
    }

    pub async fn submit_uid_sign_result_call_bytes(
        &self,
        cid: u32,
        uid: Vec<u8>,
        pk: Vec<u8>,
        sig: Vec<u8>,
        fork_id: u8,
        signature: Vec<u8>,
    ) -> Result<Vec<u8>, String> {
        let call = crate::node::tx()
            .channel()
            .submit_uid_sign_result(cid, uid, pk, sig, fork_id, signature);
        self.client
            .unsigned_tx_encode_to_bytes(call)
            .await
            .map_err(|e| e.to_string())
    }

    pub async fn request_to_sign_forced_withdrawal(
        &self,
        tx_nonce: u128,
        msg: Vec<u8>,
        watch_res: bool,
        nonce: Option<u32>,
    ) -> Result<Hash, String> {
        let call = crate::node::tx()
            .channel()
            .sign_forced_withdrawal(tx_nonce, msg);
        if watch_res {
            self.client
                .submit_extrinsic_with_signer_and_watch(call, nonce)
                .await
                .map_err(|e| e.to_string())
        } else {
            self.client
                .submit_extrinsic_with_signer_without_watch(call, nonce)
                .await
                .map_err(|e| e.to_string())
        }
    }

    pub async fn finish_forced_withdrawal_result(
        &self,
        cid: u32,
        tx_nonce: u128,
        sender_pk: Vec<u8>,
        sender_sig: Vec<u8>,
        cmt_sig: Vec<u8>,
        fork_id: u8,
    ) -> Result<Hash, String> {
        let call = crate::node::tx()
            .channel()
            .finish_forced_withdrawal(cid, tx_nonce, sender_pk, sender_sig, cmt_sig, fork_id);
        self.client
            .submit_extrinsic_without_signer(call)
            .await
            .map_err(handle_custom_error)
    }

    pub async fn finish_forced_withdrawal_result_call_bytes(
        &self,
        cid: u32,
        tx_nonce: u128,
        sender_pk: Vec<u8>,
        sender_sig: Vec<u8>,
        cmt_sig: Vec<u8>,
        fork_id: u8,
    ) -> Result<Vec<u8>, String> {
        let call = crate::node::tx()
            .channel()
            .finish_forced_withdrawal(cid, tx_nonce, sender_pk, sender_sig, cmt_sig, fork_id);
        self.client
            .unsigned_tx_encode_to_bytes(call)
            .await
            .map_err(|e| e.to_string())
    }
}
