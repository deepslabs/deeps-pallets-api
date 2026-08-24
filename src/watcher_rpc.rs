#![allow(clippy::type_complexity)]

//! relay service for key server.
use crate::{
    no_prefix,
    node::runtime_types::{
        ethereum::transaction::{
            eip1559::EIP1559Transaction, eip2930::TransactionSignature, legacy::TransactionAction, TransactionV3 as Transaction,
        },
        pallet_facility::pallet::DIdentity,
        pallet_mining::types::{DeviceMode, MonitorType, OnChainPayload, Purpose},
    },
    NodeRpc,
};
use codec::Encode;

use crate::NodeClient;
use precompile_utils::{prelude::UnboundedBytes, solidity::codec::Writer as EvmDataWriter};
use sp_core::H256;
use subxt::utils::H160;

/// keccak_256("submitTxSignResult(bytes[],bytes[],uint256,uint256,bytes32,bytes[])".as_bytes())[..4]
pub const REPORT_RESULT_SELECTOR: [u8; 4] = [118, 72, 134, 178];
/// keccak_256("submitGroupedTxSignResult(bytes,bytes,uint256,uint256,bytes32,bytes,uint16[])".as_bytes())[..4]
pub const REPORT_GROUPED_RESULT_SELECTOR: [u8; 4] = [16, 246, 155, 4];
/// keccak_256("importNewTx(uint256,uint256,bytes[],uint256,bytes[],bytes[],bytes[],uint256)".as_bytes())[..4]
pub const SUBMIT_TRANSACTION_SELECTOR: [u8; 4] = [58, 164, 61, 2];
/// keccak_256("joinOrExitServiceUnsigned(bytes[],uint256,bytes[],bytes[])".as_bytes())[..4]
pub const JOIN_OR_EXIT_SERVICE_UNSIGNED_SELECTOR: [u8; 4] = [99, 254, 70, 76];
pub async fn call_register_v2(
    sub_client: &NodeClient,
    config_owner: &str,
    did: (u16, Vec<u8>),
    report: Vec<u8>,
    identity: Vec<u8>,
    device_mode: DeviceMode,
    monitor_type: MonitorType,
    signature: Vec<u8>,
) -> Result<String, String> {
    let (version, _pk) = did;
    let owner = hex::decode(no_prefix(config_owner)).map_err(|e| e.to_string())?;
    let mut owner_bytes = [0u8; 20];
    owner_bytes.copy_from_slice(&owner);
    match sub_client
        .submit()
        .mining()
        .register_device(
            crate::node::runtime_types::fp_account::AccountId20(owner_bytes),
            report,
            version,
            identity,
            device_mode,
            monitor_type,
            signature,
        )
        .await
    {
        Ok(hash) => Ok("0x".to_string() + &hex::encode(hash.0)),
        Err(e) => Err(e),
    }
}
pub async fn call_heartbeat(
    sub_client: &NodeClient,
    did: (u16, Vec<u8>),
    signature: Vec<u8>,
    proof: Vec<u8>,
    session: u32,
    enclave: Vec<u8>,
) -> Result<String, String> {
    let did = DIdentity {
        version: did.0,
        pk: did.1,
    };
    let payload = OnChainPayload {
        did,
        proof,
        session,
        signature,
        enclave,
    };
    match sub_client.submit().mining().im_online(payload).await {
        Ok(hash) => Ok("0x".to_string() + &hex::encode(hash.0)),
        Err(e) => Err(e),
    }
}

pub async fn query_session_and_challenge(
    sub_client: &NodeClient,
    did: (u16, Vec<u8>),
) -> Result<Option<(u32, Vec<u8>)>, String> {
    let did = DIdentity {
        version: did.0,
        pk: did.1,
    };
    let (devices, session) = sub_client
        .query()
        .mining()
        .working_devices(None, None)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("no working device".to_string())?;
    let res = if devices.contains(&(did, false)) {
        match sub_client
            .query()
            .mining()
            .challenges(session, None)
            .await
            .map_err(|e| e.to_string())?
        {
            Some(challenges) => Some((session, challenges.encode())),
            None => None,
        }
    } else {
        None
    };
    Ok(res)
}

pub async fn report_result_by_evm(
    sub_client: &NodeClient,
    pk: Vec<u8>,
    sig: Vec<u8>,
    cid: u32,
    fork_id: u8,
    hash: H256,
    signature: Vec<u8>,
    call_bytes: bool,
    grouped_index: Option<Vec<u16>>,
) -> Result<Vec<u8>, String> {
    let input = build_report_result_calldata(pk, sig, cid, fork_id, hash, signature, grouped_index);

    let chain_id = sub_client
        .query()
        .ethereum()
        .evm_chain_id(None)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("get evm chain failed".to_string())?;
    let zero_u256 = [0u64; 4];
    let transaction = Transaction::EIP1559(EIP1559Transaction {
        chain_id,
        nonce: crate::node::runtime_types::primitive_types::U256(zero_u256),
        max_priority_fee_per_gas: crate::node::runtime_types::primitive_types::U256(zero_u256),
        max_fee_per_gas: crate::node::runtime_types::primitive_types::U256(zero_u256),
        gas_limit: crate::node::runtime_types::primitive_types::U256([50000000u64, 0, 0, 0]),
        // channel precompile contract address
        action: TransactionAction::Call(H160::from_low_u64_be(1104)),
        value: crate::node::runtime_types::primitive_types::U256(zero_u256),
        input,
        access_list: vec![],
        signature: TransactionSignature {
            odd_y_parity: Default::default(),
            r: subxt::utils::H256::from_low_u64_be(1),
            s: subxt::utils::H256::from_low_u64_be(1),
        },
    });

    if call_bytes {
        sub_client
            .submit()
            .ethereum()
            .transact_unsigned_call_bytes(transaction)
            .await
    } else {
        sub_client
            .submit()
            .ethereum()
            .transact_unsigned(transaction)
            .await
            .map(|hash| hash.0.to_vec())
    }
}

/// Build the EVM calldata for reportResult / reportGroupedResult without needing a node client.
/// This pure function is useful for testing the ABI encoding logic.
pub fn build_report_result_calldata(
    pk: Vec<u8>,
    sig: Vec<u8>,
    cid: u32,
    fork_id: u8,
    hash: H256,
    signature: Vec<u8>,
    grouped_index: Option<Vec<u16>>,
) -> Vec<u8> {
    let writer = if let Some(grouped_index) = grouped_index {
        EvmDataWriter::new_with_selector(u32::from_be_bytes(REPORT_GROUPED_RESULT_SELECTOR))
            .write(UnboundedBytes::from(pk))
            .write(UnboundedBytes::from(sig))
            .write(cid)
            .write(fork_id)
            .write(hash)
            .write(UnboundedBytes::from(signature))
            .write(grouped_index)
    } else {
        EvmDataWriter::new_with_selector(u32::from_be_bytes(REPORT_RESULT_SELECTOR))
            .write(UnboundedBytes::from(pk))
            .write(UnboundedBytes::from(sig))
            .write(cid)
            .write(fork_id)
            .write(hash)
            .write(UnboundedBytes::from(signature))
    };
    writer.build()
}

pub async fn join_or_exit_service_unsigned_by_evm(
    sub_client: &NodeClient,
    id: Vec<u8>,
    msg: Vec<u8>,
    signature: Vec<u8>,
    purpose: Purpose,
) -> Result<String, String> {
    // build writer with select
    let writer = EvmDataWriter::new_with_selector(u32::from_be_bytes(
        JOIN_OR_EXIT_SERVICE_UNSIGNED_SELECTOR,
    ))
    .write(UnboundedBytes::from(id))
    .write(purpose as u8)
    .write(UnboundedBytes::from(msg))
    .write(UnboundedBytes::from(signature));

    let input = writer.build();

    let chain_id = sub_client
        .query()
        .ethereum()
        .evm_chain_id(None)
        .await
        .map_err(|e| e.to_string())?
        .ok_or("get evm chain failed".to_string())?;
    let zero_u256 = [0u64; 4];
    let transaction = Transaction::EIP1559(EIP1559Transaction {
        chain_id,
        nonce: crate::node::runtime_types::primitive_types::U256(zero_u256),
        max_priority_fee_per_gas: crate::node::runtime_types::primitive_types::U256(zero_u256),
        max_fee_per_gas: crate::node::runtime_types::primitive_types::U256(zero_u256),
        gas_limit: crate::node::runtime_types::primitive_types::U256([50000000u64, 0, 0, 0]),
        // mining precompile contract address
        action: TransactionAction::Call(H160::from_low_u64_be(1101)),
        value: crate::node::runtime_types::primitive_types::U256(zero_u256),
        input,
        access_list: vec![],
        signature: TransactionSignature {
            odd_y_parity: Default::default(),
            r: subxt::utils::H256::from_low_u64_be(1),
            s: subxt::utils::H256::from_low_u64_be(1),
        },
    });

    sub_client
        .submit()
        .ethereum()
        .transact_unsigned(transaction)
        .await
        .map(|hash| "0x".to_string() + &hex::encode(hash.0))
}

pub async fn query_current_block_number(sub_client: &NodeClient) -> Result<u32, String> {
    sub_client
        .client
        .read()
        .await
        .blocks()
        .at_latest()
        .await
        .map(|block| block.number())
        .map_err(|e| e.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use sp_core::H256;
    use crate::{Secp256k1Signer, SecretKey};

    #[tokio::test]
    async fn test_report_result_by_call_bytes() {
        std::env::set_var("RUST_LOG", "debug");
        env_logger::init();
        use crate::NodeClient;

        let url = "ws://127.0.0.1:9933".to_string();
        let sk_bytes =
            hex::decode("5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133")
                .unwrap(); // alice
        let sk = SecretKey::parse_slice(&sk_bytes).unwrap();
        let signer = Secp256k1Signer::new(sk);
        let client = NodeClient::new_from_signer(&url, Some(signer), None, Some(20))
            .await
            .unwrap();
        let call_bytes = report_result_by_evm(
            &client,
            vec![0u8; 33],
            vec![0u8; 65],
            2,
            1,
            H256::from_low_u64_be(123456),
            vec![0u8; 65],
            true,
            None,
        ).await.unwrap();
        log::info!("call_bytes: {:?}", hex::encode(&call_bytes));
        let res = client
            .submit_extrinsic_without_signer_from_bytes(call_bytes)
            .await
            .map_err(|e| e.to_string());
        log::info!("submit res: {res:?}");
    }
}
