use crate::node::runtime_types::ethereum::transaction::{
    EIP1559Transaction, TransactionAction, TransactionV2 as EvmTransaction,
};
use crate::{Secp256k1Signer, SecretKey};
use anyhow::Result;
use codec::{Compact, Encode};
use node_primitives::AccountId20;
use subxt::utils::H256 as Hash;
use std::collections::HashMap;
use std::sync::Arc;
use std::time::Instant;
use subxt::config::extrinsic_params::BaseExtrinsicParamsBuilder;
use subxt::config::{
    polkadot::{PolkadotExtrinsicParams, U256 as EthU256},
    substrate::{BlakeTwo256, SubstrateHeader},
};
use subxt::tx::{Signer, SubmittableExtrinsic};
use subxt::{
    error::RpcError,
    storage::{address::Yes, StorageAddress, StorageKey},
    tx::{TxPayload, TxProgress},
    Config, Error, JsonRpseeError, OnlineClient,
};
use tokio::sync::RwLock;

#[derive(Clone, Debug)]
pub enum NodeConfig {}

impl Config for NodeConfig {
    type Index = u32;
    type Hash = subxt::utils::H256;
    type AccountId = AccountId20;
    type Address = sp_runtime::MultiAddress<AccountId20, ()>;
    type Signature = node_primitives::EthereumSignature;
    type Hasher = BlakeTwo256;
    type Header = SubstrateHeader<u32, BlakeTwo256>;
    type ExtrinsicParams = PolkadotExtrinsicParams<Self>;
}

#[derive(Clone)]
pub struct SubClient<C: Config, P: Signer<C> + Clone> {
    pub ws_url: String,
    pub signer: Option<P>,
    pub client: Arc<RwLock<OnlineClient<C>>>,
    pub inner_nonce: Arc<RwLock<u32>>,
    // number of cache, will re-submit call if 'call_cache' length up to it.
    pub cache_size_for_call: u32,
    // call cache with target nonce, ture value for 'node' param means the tx is submitted by evm, 'Vec<u8>' is input for evm tx, 'u128' is tx tip for priority.
    pub call_cache:
        Arc<RwLock<HashMap<u32, (Box<dyn TxPayload + Send + Sync>, bool, Vec<u8>, u128)>>>,
    // milliseconds, default 10000 millis(10 seconds)
    pub warn_time: u128,
}

impl SubClient<NodeConfig, Secp256k1Signer<NodeConfig>> {
    pub async fn new(
        url: &str,
        id: &str,
        password_override: Option<String>,
        warn_time: Option<u128>,
        cache_size_for_call: Option<u32>,
    ) -> SubClient<NodeConfig, Secp256k1Signer<NodeConfig>> {
        let password_override = password_override.unwrap_or("".to_string());
        let phase = id.to_owned() + &password_override;
        let seed = sp_core::keccak_256(phase.as_bytes());
        let signer = Secp256k1Signer::new(
            SecretKey::parse(&seed).expect("phase sk from seed should successfully"),
        );
        let subxt_client = OnlineClient::<NodeConfig>::from_url(url).await.unwrap();
        let chain_nonce = subxt_client
            .tx()
            .account_nonce(signer.account_id())
            .await
            .unwrap() as u32;
        SubClient {
            ws_url: url.to_string(),
            signer: Some(signer),
            client: Arc::new(RwLock::new(subxt_client)),
            inner_nonce: Arc::new(RwLock::new(chain_nonce)),
            cache_size_for_call: cache_size_for_call.unwrap_or(10),
            call_cache: Arc::new(RwLock::new(HashMap::new())),
            warn_time: warn_time.unwrap_or(10000),
        }
    }

    pub async fn new_from_ecdsa_sk(
        url: String,
        sk: Option<String>,
        warn_time: Option<u128>,
        cache_size_for_call: Option<u32>,
    ) -> Result<SubClient<NodeConfig, Secp256k1Signer<NodeConfig>>, String> {
        let mut chain_nonce = 0;
        let subxt_client = OnlineClient::<NodeConfig>::from_url(&url)
            .await
            .map_err(|e| e.to_string())?;
        let signer = if let Some(sk) = sk {
            let sk =
                hex::decode(sk.strip_prefix("0x").unwrap_or(&sk)).map_err(|e| e.to_string())?;
            let signer =
                Secp256k1Signer::new(SecretKey::parse_slice(&sk).map_err(|e| e.to_string())?);
            chain_nonce = subxt_client
                .tx()
                .account_nonce(signer.account_id())
                .await
                .unwrap() as u32;
            Some(signer)
        } else {
            None
        };
        Ok(SubClient {
            ws_url: url,
            signer,
            client: Arc::new(RwLock::new(subxt_client)),
            inner_nonce: Arc::new(RwLock::new(chain_nonce)),
            cache_size_for_call: cache_size_for_call.unwrap_or(10),
            call_cache: Arc::new(RwLock::new(HashMap::new())),
            warn_time: warn_time.unwrap_or(10000),
        })
    }

    pub async fn submit_extrinsic_with_signer_and_watch<Call: TxPayload + 'static + Send + Sync>(
        &self,
        call: Call,
        nonce: Option<u32>,
    ) -> Result<Hash, Error> {
        let call = Box::new(call);
        let timer = Instant::now();
        self.check_client_runtime_version_and_update().await?;

        let mut inner_nonce = self.inner_nonce.write().await;
        let mut call_cache = self.call_cache.write().await;
        let client = self.client.read().await;
        let signer = self
            .signer
            .as_ref()
            .ok_or_else(|| Error::Other("empty sk to sign and submit tx".to_string()))?;
        let account_id = signer.account_id();

        let target_nonce = if let Some(nonce) = nonce {
            nonce
        } else {
            let chain_nonce = client.tx().account_nonce(account_id).await? as u32;
            // clear cache for lower nonce, retain 10 nonce due to 'chain_nonce' can roll back
            let oldest_nonce = std::cmp::max(chain_nonce, 10);
            let old_nonce = call_cache
                .keys()
                .filter(|v| v < &&(oldest_nonce - 10))
                .cloned()
                .collect::<Vec<_>>();
            for key in old_nonce {
                log::trace!(target: "subxt::call_cache", "remove key {:?}", key);
                call_cache.remove(&key);
            }
            if chain_nonce >= *inner_nonce {
                chain_nonce
            } else {
                // Some errors occurred. ie. some tx with nonce not submit to chain seccessfully.
                if *inner_nonce - chain_nonce > self.cache_size_for_call {
                    log::warn!(target: "subxt", "Some errors occurred to nonce inner {}, chain {}", *inner_nonce, chain_nonce);
                    for key in chain_nonce..*inner_nonce {
                        if let Some((inner_call, by_evm, _, _)) = call_cache.get(&key) {
                            let tx = if *by_evm {
                                client.tx().create_unsigned(inner_call)?
                            } else {
                                client.tx().create_signed_with_nonce(
                                    inner_call,
                                    signer,
                                    key,
                                    Default::default(),
                                )?
                            };
                            let tx_hash = tx.submit().await;
                            log::warn!(target: "subxt", "re-submit call with nonce: {}, res: {:?}", key, tx_hash);
                        } else {
                            log::warn!(target: "subxt", "re-submit call not find nonce: {} in cache", key);
                        }
                    }
                }
                *inner_nonce
            }
        };
        let tx: subxt::tx::SubmittableExtrinsic<NodeConfig, OnlineClient<NodeConfig>> = client
            .tx()
            .create_signed_with_nonce(&call, signer, target_nonce, Default::default())?;
        let tx_hash = match tx.submit_and_watch().await?.wait_for_in_block().await {
            Ok(tx) => {
                log::debug!(target: "subxt::nonce", "inner_nonce {}, insert cache for nonce: {}", target_nonce + 1, target_nonce);
                *inner_nonce = target_nonce + 1;
                // update call_cache
                call_cache.insert(target_nonce, (call, false, vec![], 0));
                tx.wait_for_success().await?.extrinsic_hash()
            }
            Err(e) => return Err(e),
        };
        if timer.elapsed().as_millis() > self.warn_time {
            log::warn!(target: "subxt", "submit_extrinsic_with_signer_and_watch exceed warn_time: {} millis", timer.elapsed().as_millis());
        }
        Ok(tx_hash)
    }

    pub async fn submit_extrinsic_with_signer_without_watch<
        Call: TxPayload + 'static + Send + Sync,
    >(
        &self,
        call: Call,
        nonce: Option<u32>,
    ) -> Result<<NodeConfig as Config>::Hash, Error> {
        let call = Box::new(call);
        let timer = Instant::now();
        self.check_client_runtime_version_and_update().await?;

        let mut inner_nonce = self.inner_nonce.write().await;
        let mut call_cache = self.call_cache.write().await;
        let client = self.client.read().await;
        let signer = self
            .signer
            .as_ref()
            .ok_or_else(|| Error::Other("empty sk to sign and submit tx".to_string()))?;
        let account_id = signer.account_id();

        let target_nonce = if let Some(nonce) = nonce {
            nonce
        } else {
            let chain_nonce = client.tx().account_nonce(account_id).await? as u32;
            // clear cache for lower nonce
            let old_nonce = call_cache
                .keys()
                .filter(|v| v < &&chain_nonce)
                .cloned()
                .collect::<Vec<_>>();
            for key in old_nonce {
                log::trace!(target: "subxt::call_cache", "remove key {:?}", key);
                call_cache.remove(&key);
            }
            if chain_nonce >= *inner_nonce {
                chain_nonce
            } else {
                // Some errors occurred. ie. some tx with nonce not submit to chain seccessfully.
                if *inner_nonce - chain_nonce >= self.cache_size_for_call {
                    log::warn!(target: "subxt", "Some errors occurred to nonce inner {}, chain {}", *inner_nonce, chain_nonce);
                    for key in chain_nonce..*inner_nonce {
                        if let Some((inner_call, by_evm, input, tip)) = call_cache.get_mut(&key) {
                            let tx = if *by_evm {
                                let mut eip1995_tx =
                                    <ethereum::EIP1559Transaction as codec::Decode>::decode(
                                        &mut input.as_slice(),
                                    )?;
                                eip1995_tx.max_priority_fee_per_gas = eip1995_tx
                                    .max_priority_fee_per_gas
                                    + EthU256::from(*tip + 100u128);
                                let evm_tx = self
                                    .build_eip1559_tx_to_v2(eip1995_tx)
                                    .map_err(|e| Error::Other(e))?;
                                let evm_call = crate::node::tx().ethereum().transact(evm_tx);
                                client.tx().create_unsigned(&evm_call)?
                            } else {
                                client.tx().create_signed_with_nonce(
                                    inner_call,
                                    signer,
                                    key,
                                    BaseExtrinsicParamsBuilder::new().tip(*tip + 100),
                                )?
                            };
                            let tx_hash = tx.submit().await;
                            log::warn!(target: "subxt", "re-submit call with nonce: {}, tip: {:?}, res: {:?}", key, *tip + 100, tx_hash);
                            //update tip
                            *tip += 100;
                        } else {
                            log::warn!(target: "subxt", "re-submit call not find nonce: {} in cache", key);
                        }
                    }
                }
                *inner_nonce
            }
        };
        let tx = client.tx().create_signed_with_nonce(
            &call,
            signer,
            target_nonce,
            Default::default(),
        )?;
        let tx_hash = match tx.submit().await {
            Ok(tx) => {
                log::debug!(target: "subxt::nonce", "inner_nonce {}, insert cache for nonce: {}", target_nonce + 1, target_nonce);
                *inner_nonce = target_nonce + 1;
                // update call_cache
                call_cache.insert(target_nonce, (call, false, vec![], 0));
                tx
            }
            Err(e) => return Err(e),
        };
        if timer.elapsed().as_millis() > self.warn_time {
            log::warn!(target: "subxt", "submit_extrinsic_with_signer_and_watch exceed warn_time: {} millis", timer.elapsed().as_millis());
        }
        Ok(tx_hash)
    }

    pub async fn signed_tx_encode_to_bytes<Call: TxPayload + 'static + Send + Sync>(
        &self,
        call: Call,
        nonce: Option<u32>,
    ) -> Result<Vec<u8>, Error> {
        let inner_nonce = self.inner_nonce.read().await;
        let mut call_cache = self.call_cache.write().await;
        let client = self.client.read().await;
        let signer = self
            .signer
            .as_ref()
            .ok_or_else(|| Error::Other("empty sk to sign and submit tx".to_string()))?;
        let account_id = signer.account_id();

        let target_nonce = if let Some(nonce) = nonce {
            nonce
        } else {
            let chain_nonce = client.tx().account_nonce(account_id).await? as u32;
            // clear cache for lower nonce
            let old_nonce = call_cache
                .keys()
                .filter(|v| v < &&chain_nonce)
                .cloned()
                .collect::<Vec<_>>();
            for key in old_nonce {
                log::trace!(target: "subxt::call_cache", "remove key {:?}", key);
                call_cache.remove(&key);
            }
            if chain_nonce >= *inner_nonce {
                chain_nonce
            } else {
                // Some errors occurred. ie. some tx with nonce not submit to chain seccessfully.
                if *inner_nonce - chain_nonce >= self.cache_size_for_call {
                    log::warn!(target: "subxt", "Some errors occurred to nonce inner {}, chain {}", *inner_nonce, chain_nonce);
                    for key in chain_nonce..*inner_nonce {
                        if let Some((inner_call, by_evm, input, tip)) = call_cache.get_mut(&key) {
                            let tx = if *by_evm {
                                let mut eip1995_tx =
                                    <ethereum::EIP1559Transaction as codec::Decode>::decode(
                                        &mut input.as_slice(),
                                    )?;
                                eip1995_tx.max_priority_fee_per_gas = eip1995_tx
                                    .max_priority_fee_per_gas
                                    + EthU256::from(*tip + 100u128);
                                let evm_tx = self
                                    .build_eip1559_tx_to_v2(eip1995_tx)
                                    .map_err(|e| Error::Other(e))?;
                                let evm_call = crate::node::tx().ethereum().transact(evm_tx);
                                client.tx().create_unsigned(&evm_call)?
                            } else {
                                client.tx().create_signed_with_nonce(
                                    inner_call,
                                    signer,
                                    key,
                                    BaseExtrinsicParamsBuilder::new().tip(*tip + 100),
                                )?
                            };
                            let tx_hash = tx.submit().await;
                            log::warn!(target: "subxt", "re-submit call with nonce: {}, tip: {:?}, res: {:?}", key, *tip + 100, tx_hash);
                            //update tip
                            *tip += 100;
                        } else {
                            log::warn!(target: "subxt", "re-submit call not find nonce: {} in cache", key);
                        }
                    }
                }
                *inner_nonce
            }
        };

        // 1. Validate this call against the current node metadata if the call comes
        // with a hash allowing us to do so.
        client.tx().validate(&call)?;

        // 2. Gather the "additional" and "extra" params along with the encoded call data,
        //    ready to be signed.
        let partial_signed = client.tx().create_partial_signed_with_nonce(
            &call,
            target_nonce,
            Default::default(),
        )?;

        // 3. Sign and construct an extrinsic from these details.
        let tx = partial_signed.sign(signer);

        Ok(tx.into_encoded())
    }

    pub async fn submit_extrinsic_without_signer<Call: TxPayload + 'static + Send + Sync>(
        &self,
        call: Call,
    ) -> Result<Hash, Error> {
        let timer = Instant::now();
        let client = self.client.read().await;
        let tx = client.tx().create_unsigned(&Box::new(call))?;
        let tx_hash = tx.submit().await?;
        if timer.elapsed().as_millis() > self.warn_time {
            log::warn!(target: "subxt", "submit_extrinsic_without_signer exceed warn_time: {} millis", timer.elapsed().as_millis());
        }
        Ok(tx_hash)
    }

    pub async fn unsigned_tx_encode_to_bytes<Call: TxPayload + 'static + Send + Sync>(
        &self,
        call: Call,
    ) -> Result<Vec<u8>, Error> {
        let client = self.client.read().await;
        // 1. Validate this call against the current node metadata if the call comes
        // with a hash allowing us to do so.
        client.tx().validate(&call)?;

        // 2. Encode extrinsic
        let extrinsic_encoded = {
            let mut encoded_inner = Vec::new();
            // transaction protocol version (4) (is not signed, so no 1 bit at the front).
            4u8.encode_to(&mut encoded_inner);
            // encode call data after this byte.
            call.encode_call_data_to(&client.metadata(), &mut encoded_inner)?;
            // now, prefix byte length:
            let len = Compact(
                u32::try_from(encoded_inner.len()).expect("extrinsic size expected to be <4GB"),
            );
            let mut encoded = Vec::new();
            len.encode_to(&mut encoded);
            encoded.extend(encoded_inner);
            encoded
        };
        Ok(extrinsic_encoded)
    }

    pub async fn submit_extrinsic_without_signer_from_bytes(
        &self,
        call_bytes: Vec<u8>,
    ) -> Result<Hash, Error> {
        let timer = Instant::now();
        let client = self.client.read().await;
        let tx = SubmittableExtrinsic::from_bytes(client.clone(), call_bytes);
        let tx_hash = tx.submit().await?;
        if timer.elapsed().as_millis() > self.warn_time {
            log::warn!(target: "subxt", "submit_extrinsic_without_signer exceed warn_time: {} millis", timer.elapsed().as_millis());
        }
        Ok(tx_hash)
    }

    pub async fn submit_extrinsic_without_signer_and_watch<Call: TxPayload>(
        &self,
        call: Call,
    ) -> Result<TxProgress<NodeConfig, OnlineClient<NodeConfig>>, Error> {
        let timer = Instant::now();
        let client = self.client.read().await;
        let tx = client.tx().create_unsigned(&call)?;
        let tx_process = tx.submit_and_watch().await;
        if timer.elapsed().as_millis() > self.warn_time {
            log::warn!(target: "subxt", "submit_extrinsic_without_signer_and_watch exceed warn_time: {} millis", timer.elapsed().as_millis());
        }
        tx_process
    }

    pub async fn query_storage<F: StorageAddress<IsFetchable = Yes>>(
        &self,
        store_query: F,
        at_block: Option<Hash>,
    ) -> Result<Option<F::Target>, Error> {
        let timer = Instant::now();
        self.check_client_runtime_version_and_update().await?;
        let storage_client = self.client.read().await.storage();
        let res = match at_block {
            Some(block) => storage_client.at(block).fetch(&store_query).await,
            None => storage_client.at_latest().await?.fetch(&store_query).await,
        };
        if timer.elapsed().as_millis() > self.warn_time {
            log::warn!(target: "subxt", "query_storage exceed warn_time: {} millis", timer.elapsed().as_millis());
        }
        res
    }

    pub async fn query_storage_value_iter<F: StorageAddress<IsIterable = Yes> + 'static>(
        &self,
        store_query: F,
        page_sise: u32,
        at_block: Option<Hash>,
    ) -> Result<Vec<(StorageKey, F::Target)>, Error> {
        let timer = Instant::now();
        self.check_client_runtime_version_and_update().await?;
        let storage_client = self.client.read().await.storage();
        let mut iter = match at_block {
            Some(block) => {
                storage_client
                    .at(block)
                    .iter(store_query, page_sise)
                    .await?
            }
            None => {
                storage_client
                    .at_latest()
                    .await?
                    .iter(store_query, page_sise)
                    .await?
            }
        };
        let mut values = Vec::new();
        while let Some((key, value)) = iter.next().await? {
            values.push((key, value))
        }
        if timer.elapsed().as_millis() > self.warn_time {
            log::warn!(target: "subxt", "query_storage exceed warn_time: {} millis", timer.elapsed().as_millis());
        }
        Ok(values)
    }

    pub async fn query_storage_or_default<
        F: StorageAddress<IsFetchable = Yes, IsDefaultable = Yes>,
    >(
        &self,
        store_query: F,
        at_block: Option<Hash>,
    ) -> Result<F::Target, Error> {
        let timer = Instant::now();
        self.check_client_runtime_version_and_update().await?;
        let storage_client = self.client.read().await.storage();
        let res = match at_block {
            Some(block) => {
                storage_client
                    .at(block)
                    .fetch_or_default(&store_query)
                    .await
            }
            None => {
                storage_client
                    .at_latest()
                    .await?
                    .fetch_or_default(&store_query)
                    .await
            }
        };
        if timer.elapsed().as_millis() > self.warn_time {
            log::warn!(target: "subxt", "query_storage_or_default exceed warn_time: {} millis", timer.elapsed().as_millis());
        }
        res
    }

    pub async fn query_constant<Address: subxt::constants::ConstantAddress>(
        &self,
        address: Address,
    ) -> Result<Address::Target, Error> {
        let timer = Instant::now();
        self.check_client_runtime_version_and_update().await?;
        let client = self.client.read().await.constants();
        let res = client.at(&address);
        if timer.elapsed().as_millis() > self.warn_time {
            log::warn!(target: "subxt", "query_constant exceed warn_time: {} millis", timer.elapsed().as_millis());
        }
        res
    }

    pub async fn query_account_nonce(&self) -> Option<u32> {
        let timer = Instant::now();
        self.check_client_runtime_version_and_update().await.ok()?;
        let res = match self
            .client
            .read()
            .await
            .tx()
            .account_nonce(&self.account_id().await)
            .await
        {
            Ok(nonce) => Some(nonce),
            Err(_) => None,
        };
        if timer.elapsed().as_millis() > self.warn_time {
            log::warn!(target: "subxt", "query_account_nonce exceed warn_time: {} millis", timer.elapsed().as_millis());
        }
        res
    }

    pub async fn account_id(&self) -> AccountId20 {
        let timer = Instant::now();
        let res = self
            .signer
            .as_ref()
            .expect("subclient should has account")
            .account_id()
            .clone();
        if timer.elapsed().as_millis() > self.warn_time {
            log::warn!(target: "subxt", "account_id exceed warn_time: {} millis", timer.elapsed().as_millis());
        }
        res
    }

    pub fn build_eip1559_tx_to_v2(
        &self,
        tx: ethereum::EIP1559Transaction,
    ) -> Result<EvmTransaction, String> {
        let tx = ethereum::EIP1559TransactionMessage::from(tx);
        let sk = self
            .signer
            .clone()
            .ok_or("Not set node client signer")?
            .signer()
            .serialize();
        let secret = secp256k1::SecretKey::parse(&sk)
            .map_err(|e| format!("Parse node signer sk failed for: {:?}", e))?;
        let signing_message =
            secp256k1::Message::parse_slice(&tx.hash()[..]).map_err(|e| e.to_string())?;
        let (signature, recid) = secp256k1::sign(&signing_message, &secret);
        let rs = signature.serialize();
        let r = subxt::utils::H256::from_slice(&rs[0..32]);
        let s = subxt::utils::H256::from_slice(&rs[32..64]);
        Ok(EvmTransaction::EIP1559(EIP1559Transaction {
            chain_id: tx.chain_id,
            nonce: crate::node::runtime_types::primitive_types::U256(tx.nonce.0),
            max_priority_fee_per_gas: crate::node::runtime_types::primitive_types::U256(
                tx.max_priority_fee_per_gas.0,
            ),
            max_fee_per_gas: crate::node::runtime_types::primitive_types::U256(
                tx.max_fee_per_gas.0,
            ),
            gas_limit: crate::node::runtime_types::primitive_types::U256(tx.gas_limit.0),
            action: match tx.action {
                ethereum::TransactionAction::Call(addr) => TransactionAction::Call(addr),
                _ => return Err(format!("Invalid evm tx action: {:?}", tx.action)),
            },
            value: crate::node::runtime_types::primitive_types::U256(tx.value.0),
            input: tx.input,
            access_list: vec![],
            odd_y_parity: recid.serialize() != 0,
            r,
            s,
        }))
    }
}

impl<C: Config, P: Signer<C> + Clone> SubClient<C, P> {
    pub async fn new_from_signer(
        url: &str,
        signer: Option<P>,
        warn_time: Option<u128>,
        cache_size_for_call: Option<u32>,
    ) -> Result<SubClient<C, P>, Error> {
        let ws_url: url::Url = url
            .parse()
            .map_err(|_| Error::Other("parse url from string failed".to_string()))?;
        let mut fixed_ws_url = ws_url.as_str().to_string();
        if ws_url.port().is_none() {
            let mut tmp = vec![fixed_ws_url
                .strip_suffix(ws_url.path())
                .unwrap_or(&fixed_ws_url)];
            let default_port = format!(":{}", default_port(ws_url.scheme()).unwrap());
            tmp.push(&default_port);
            tmp.push(ws_url.path());
            fixed_ws_url = tmp.concat();
        }
        let subxt_client = OnlineClient::<C>::from_url(fixed_ws_url.clone()).await?;
        Ok(SubClient {
            ws_url: fixed_ws_url,
            signer,
            client: Arc::new(RwLock::new(subxt_client)),
            inner_nonce: Arc::new(RwLock::new(0)),
            cache_size_for_call: cache_size_for_call.unwrap_or(10),
            call_cache: Arc::new(RwLock::new(HashMap::new())),
            warn_time: warn_time.unwrap_or(10000),
        })
    }

    pub async fn check_client_runtime_version_and_update(&self) -> Result<(), Error> {
        let timer = Instant::now();
        let client = self.client.read().await;
        let res = match client.rpc().runtime_version(None).await {
            Ok(runtime_version) => {
                if runtime_version != client.runtime_version() {
                    log::warn!(target: "subxt", "invalid runtime version, try to rebuild client...");
                    drop(client);
                    self.rebuild_client().await
                } else {
                    Ok(())
                }
            }
            Err(e) => {
                log::warn!(target: "subxt", "rebuild client for: {:?}", e);
                drop(client);
                self.handle_error(e).await
            }
        };
        if timer.elapsed().as_millis() > self.warn_time {
            log::warn!(target: "subxt", "check_client_runtime_version_and_update exceed warn_time: {} millis", timer.elapsed().as_millis());
        }
        res
    }

    pub async fn rebuild_client(&self) -> Result<(), Error> {
        let timer = Instant::now();
        let res = match OnlineClient::<C>::from_url(&self.ws_url).await {
            Ok(client) => {
                *self.client.write().await = client;
                log::info!(target: "subxt", "rebuild client successful");
                Ok(())
            }
            Err(e) => Err(e),
        };
        if timer.elapsed().as_millis() > self.warn_time {
            log::warn!(target: "subxt", "rebuild_client exceed warn_time: {} millis", timer.elapsed().as_millis());
        }
        res
    }

    pub async fn handle_error(&self, err: Error) -> Result<(), Error> {
        return match err {
            Error::Rpc(RpcError::SubscriptionDropped) => {
                log::warn!(target: "subxt", "rebuild client for SubscriptionDropped");
                self.rebuild_client().await
            }
            Error::Rpc(RpcError::ClientError(client_err)) => {
                match client_err.downcast_ref::<JsonRpseeError>() {
                    Some(e) => match *e {
                        JsonRpseeError::RestartNeeded(_) => {
                            log::warn!(target: "subxt", "rebuild client for {:?}", e);
                            self.rebuild_client().await
                        }
                        _ => Err(Error::Rpc(RpcError::ClientError(client_err))),
                    },
                    // Not handle other error type now
                    None => Err(Error::Rpc(RpcError::ClientError(client_err))),
                }
            }
            _ => Err(err),
        };
    }
}

pub fn default_port(scheme: &str) -> Option<u16> {
    match scheme {
        "http" | "ws" => Some(80),
        "https" | "wss" => Some(443),
        "ftp" => Some(21),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_rebuild_client() {
        use crate::NodeRpc;

        let url = "ws://127.0.0.1:9944".to_string();
        let sk = "5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133".to_string();
        let client = SubClient::new_from_ecdsa_sk(url, Some(sk), None, None)
            .await
            .unwrap();
        loop {
            println!("try to query challenges");
            let res = client.query().mining().challenges(1, None).await.unwrap();
            println!("query challenges result: {:?}", res);
            std::thread::sleep(std::time::Duration::from_secs(2));
        }
    }

    #[tokio::test]
    async fn test_query_iter() {
        use crate::NodeRpc;

        let url = "wss://test-rpc-node-ws.node.network".to_string();
        let client = crate::client::SubClient::new_from_signer(&url, None, None, None)
            .await
            .unwrap();
        let res = client
            .query()
            .committee()
            .committees_iter(300, None)
            .await
            .unwrap();
        println!("res: {res:?}");
    }

    #[tokio::test]
    async fn test_query_cmt() {
        use crate::NodeRpc;

        let url = "wss://test-rpc-node-ws.node.network".to_string();
        let client = crate::client::SubClient::new_from_signer(&url, None, None, None)
            .await
            .unwrap();

        for i in 1u32..426 {
            let res = client
                .query()
                .committee()
                .committees(i, None)
                .await
                .unwrap();
            println!("res: {res:?}");
        }
    }

    #[tokio::test]
    async fn test_query_btc_committee_type_iter() {
        use crate::NodeRpc;

        let url = "ws://127.0.0.1:9933".to_string();
        let client = crate::client::SubClient::new_from_signer(&url, None, None, None)
            .await
            .unwrap();
        let res = client
            .query()
            .channel()
            .btc_committee_type_iter(300, None)
            .await
            .unwrap();
        println!("res: {res:?}");
        let res = client
            .query()
            .channel()
            .channel_mapping_tick_iter(300, None)
            .await
            .unwrap();
        println!("res: {res:?}");
        println!("hex: {:?}", hex::encode([240, 159, 165, 154]));
    }

    #[tokio::test]
    async fn test_nonce_roll_back() {
        std::env::set_var("RUST_LOG", "debug");
        env_logger::init();
        use crate::NodeClient;
        use std::str::FromStr;

        let url = "ws://127.0.0.1:9933".to_string();
        let sk_bytes = hex::decode("5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133").unwrap();
        let sk = SecretKey::parse_slice(&sk_bytes).unwrap();
        let signer = Secp256k1Signer::new(sk);
        let client = NodeClient::new_from_signer(&url, Some(signer), None, Some(20))
            .await
            .unwrap();
        let account = AccountId20::from_str("0x89Bdaf4AC10bC9d497BCa9a5cc37972026146E0E").unwrap();
        let dst = crate::node::runtime_types::fp_account::AccountId20(account.0);

        for i in 0..200 {
            log::info!("index: {i}");
            let call = crate::node::tx()
                .balances()
                .transfer_keep_alive(dst.clone().into(), 100000);
            let res = client
                .submit_extrinsic_with_signer_without_watch(call, None)
                .await
                .map_err(|e| e.to_string());
            log::info!("submit res: {res:?}");
        }
    }

    #[tokio::test]
    async fn test_submit_tx_by_call_bytes() {
        std::env::set_var("RUST_LOG", "debug");
        env_logger::init();
        use crate::NodeClient;
        use std::str::FromStr;

        let url = "ws://127.0.0.1:9944".to_string();
        let sk_bytes =
            hex::decode("5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133")
                .unwrap(); // alice
        let sk = SecretKey::parse_slice(&sk_bytes).unwrap();
        let signer = Secp256k1Signer::new(sk);
        let client = NodeClient::new_from_signer(&url, Some(signer), None, Some(20))
            .await
            .unwrap();
        let account = AccountId20::from_str("0x89Bdaf4AC10bC9d497BCa9a5cc37972026146E0E").unwrap();
        let dst = crate::node::runtime_types::fp_account::AccountId20(account.0);
        let call = crate::node::tx()
            .balances()
            .transfer_keep_alive(dst.clone().into(), 100000);
        let call_bytes = client.signed_tx_encode_to_bytes(call, None).await.unwrap();
        let res = client
            .submit_extrinsic_without_signer_from_bytes(call_bytes)
            .await
            .map_err(|e| e.to_string());
        log::info!("submit res: {res:?}");
    }

    #[tokio::test]
    async fn test_submit_extrinsic_by_evm() {
        std::env::set_var("RUST_LOG", "debug");
        env_logger::init();
        use crate::NodeClient;
        use crate::monitor_rpc;
        use crate::types::{ExtrinsicData, NeedSignedExtrinsic, PreparedCrossTransactionData};
        use chain_bridge::chain::ChainType;
        use node_primitives::AccountId20;
        use std::sync::Arc;
        use tokio::sync::Semaphore;

        let url = "ws://127.0.0.1:9933".to_string();

        // Generate multiple accounts by deriving from the base Alice key.
        // Each account gets its own NodeClient so nonce/cache tracking is independent.
        let base_sk_bytes =
            hex::decode("5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133")
                .unwrap(); // alice
        let num_accounts: u8 = 1;
        let mut clients: Vec<NodeClient> = Vec::new();
        for i in 0..num_accounts {
            let mut sk_bytes = base_sk_bytes.clone();
            // Derive distinct keys by XORing the first byte with the account index
            sk_bytes[0] ^= i;
            let sk = SecretKey::parse_slice(&sk_bytes).unwrap();
            let signer = Secp256k1Signer::<NodeConfig>::new(sk);
            let account = signer.account_id().clone();
            let client =
                NodeClient::new_from_signer(&url, Some(signer), None, Some(500))
                    .await
                    .unwrap();
            log::info!(
                "account[{}] = {}",
                i,
                hex::encode(AccountId20::from(account).0)
            );
            clients.push(client);
        }

        // Build test cross-transaction data
        let make_tx = |i: u32| NeedSignedExtrinsic {
            id: i,
            data: ExtrinsicData::PreparedCrossTransaction(PreparedCrossTransactionData {
                channel_id: 0,
                cid: 0,
                uid: vec![i as u8; 32],
                msg: vec![i as u8; 64],
                chain_type: ChainType::Eth,
                from: vec![0u8; 20],
                to: vec![0u8; 20],
                amount: vec![0u8; 32],
            }),
        };

        // Limit concurrent RPC requests to avoid exhausting the jsonrpsee request slots
        let semaphore = Arc::new(Semaphore::new(50));

        // Concurrently submit EVM transactions across multiple accounts (round-robin)
        let mut handles = Vec::new();
        let total_tx = 4000;
        for i in 0..total_tx {
            let ext = make_tx(i);
            let account_idx = (i as usize) % clients.len();
            let sub_client = clients[account_idx].clone();
            let permit = semaphore.clone();
            handles.push(tokio::spawn(async move {
                let _permit = permit.acquire().await;
                let res = monitor_rpc::submit_extrinsic_by_evm(&sub_client, ext).await;
                log::info!("evm tx {}/{} submit res: {:?}", account_idx, i, res);
                res
            }));
        }

        let mut success_count = 0;
        let mut fail_count = 0;
        for handle in handles {
            match handle.await {
                Ok(Ok(hash)) => {
                    log::info!("evm tx success: {}", hash);
                    success_count += 1;
                }
                Ok(Err(e)) => {
                    log::error!("evm tx failed: {}", e);
                    fail_count += 1;
                }
                Err(e) => {
                    log::error!("task panicked: {}", e);
                    fail_count += 1;
                }
            }
        }
        log::info!(
            "evm batch result: success={}, fail={}, total={}",
            success_count,
            fail_count,
            total_tx
        );

        for (i, client) in clients.iter().enumerate() {
            log::info!(
                "account[{}] inner_nonce: {}",
                i,
                *client.inner_nonce.read().await
            );
        }
    }
}
