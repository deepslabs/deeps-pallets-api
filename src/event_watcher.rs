//! EventWatcher for DeepSafe node witch DeepSafeSubClient.
use crate::{NodeConfig, DeepSafeSubClient as SubClient};
use deeps_node_primitives::Hash;
use std::{cmp::Ordering, collections::HashMap};
use subxt::events::EventDetails;
use subxt::blocks::BlockRef;
use subxt::Config;
use tokio::sync::mpsc::Sender;
use crate::query::system::{block_hash, latest_block_hash};

#[derive(Copy, Clone, Debug, Default, PartialEq)]
pub enum WatcherMode {
    #[default]
    Both,
    Latest,
    Finalized,
}

#[derive(Clone, Debug)]
pub enum EventFilter {
    // pallet names
    Pallets(Vec<String>),
    // pallet -> event_names
    Events(HashMap<String, Vec<String>>),
}

#[derive(Clone)]
pub struct EventWatcher {
    log_target: String,
    client: SubClient,
    handler: Sender<(WatcherMode, u32, Hash, Vec<EventDetails<NodeConfig>>)>,
    pub filter: Option<EventFilter>,
    pub latest: u32,
    pub finalized: u32,
}

impl EventWatcher {
    pub fn new(
        log_target: &str,
        client: SubClient,
        handler: Sender<(WatcherMode, u32, Hash, Vec<EventDetails<NodeConfig>>)>,
    ) -> Self {
        EventWatcher {
            log_target: log_target.to_string(),
            client,
            handler,
            filter: None,
            latest: 0,
            finalized: 0,
        }
    }

    pub fn set_filter(&mut self, filter: Option<EventFilter>) {
        self.filter = filter;
    }

    pub async fn initialize(&mut self) {
        // initialize latest block number
        loop {
            match get_block_number(self.client.clone(), None).await {
                Ok(block_number) => {
                    self.latest = block_number;
                    break;
                }
                Err(e) => log::error!(target: &self.log_target, "initialize latest block: {e:?}"),
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        // initialize finalized block number
        loop {
            match get_block_hash(self.client.clone(), WatcherMode::Finalized).await {
                Ok(hash) => match get_block_number(self.client.clone(), Some(hash)).await {
                    Ok(block_number) => {
                        log::info!(target: &self.log_target, "init finalized block_number: {block_number:?}");
                        self.finalized = block_number;
                        log::info!(target: &self.log_target, "Initialize event_watcher with latest_block {}, finalized block {}", self.latest, self.finalized);
                        break;
                    }
                    Err(e) => {
                        log::error!(target: &self.log_target, "initialize finalized block: {e:?}")
                    }
                },
                Err(e) => {
                    log::error!(target: &self.log_target, "initialize finalized block: {e:?}")
                }
            }
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
        }
        #[cfg(feature = "telemetry")]
        {
            deeps_telemetry_client::set_best_block_number(self.latest);
            deeps_telemetry_client::set_finalized_block_number(self.finalized);
            deeps_telemetry_client::set_handled_block_number(self.finalized);
        }
    }

    pub fn run(mut self, mode: WatcherMode) {
        tokio::spawn(async move {
            log::info!(target: &self.log_target, "Start watching blocks by url: {}......", &self.client.ws_url);
            loop {
                if matches!(mode, WatcherMode::Latest | WatcherMode::Both) {
                    match get_block_number(self.client.clone(), None).await {
                        Ok(current_number) => {
                            // delay one block for 'frame_system::block_hash'
                            let current_number = current_number - 1;
                            log::debug!(target: &self.log_target, "get latest block number: {current_number:?}");
                            match self.latest.cmp(&current_number) {
                                Ordering::Less => {
                                    log::trace!(target: &self.log_target, "handle latest block from {:?} to {current_number}", self.latest);
                                    self.handle_blocks_events(
                                        self.latest + 1,
                                        current_number,
                                        WatcherMode::Latest,
                                    )
                                        .await;
                                    self.latest = current_number;
                                }
                                Ordering::Equal => {
                                    log::debug!(target: &self.log_target, "caught up with the best latest block height: {current_number:?}")
                                }
                                Ordering::Greater => {
                                    log::debug!(target: &self.log_target, "latest block height is rolled back, from {:?} to {current_number:?}", self.latest)
                                }
                            }
                        },
                        Err(e) => log::error!(target: &self.log_target, "get latest block: {e:?}"),
                    };
                }

                if matches!(mode, WatcherMode::Finalized | WatcherMode::Both) {
                    match get_block_hash(self.client.clone(), WatcherMode::Finalized).await {
                        Ok(hash) => match get_block_number(self.client.clone(), Some(hash)).await {
                            Ok(current_number) => {
                                let current_number = current_number - 1;
                                log::debug!(target: &self.log_target, "get finalized block number: {current_number:?}");
                                match self.finalized.cmp(&current_number) {
                                    Ordering::Less => {
                                        log::trace!(target: &self.log_target, "handle finalized block from {:?} to {current_number}", self.finalized);
                                        self.handle_blocks_events(
                                            self.finalized + 1,
                                            current_number,
                                            WatcherMode::Finalized,
                                        )
                                            .await;
                                        self.finalized = current_number;
                                    }
                                    Ordering::Equal => {
                                        log::debug!(target: &self.log_target, "caught up with the best finalized block height: {current_number:?}")
                                    }
                                    Ordering::Greater => {
                                        log::warn!(target: &self.log_target, "finalized block height is rolled back, local: {:?}, chain: {current_number:?}", self.finalized)
                                    }
                                }
                            },
                            Err(e) => {
                                log::error!(target: &self.log_target, "get finalized block number err: {e:?}")
                            }
                        },
                        Err(e) => {
                            log::error!(target: &self.log_target, "get finalized block hash err: {e:?}")
                        }
                    };
                }

                #[cfg(feature = "telemetry")]
                {
                    deeps_telemetry_client::set_best_block_number(self.latest);
                    deeps_telemetry_client::set_finalized_block_number(self.finalized);
                }

                tokio::time::sleep(std::time::Duration::from_secs(3)).await;
            }
        });
    }

    /// handle blocks between [from, to]
    async fn handle_blocks_events(&self, from: u32, to: u32, mode: WatcherMode) {
        // handle block one by one
        for block in from..=to {
            match block_hash(&self.client, block, None).await {
                Ok(hash) => match hash {
                    Some(hash) => {
                        let events = match self.client.client.read().await.events().at(hash).await {
                            Ok(events) => events,
                            Err(e) => {
                                panic!("event watcher get events by block hash: {hash:?} failed for: {e:?}");
                            }
                        };
                        let events: Vec<_> = events
                            .iter()
                            .into_iter()
                            .filter_map(|event| match event {
                                Ok(event) => {
                                    if let Some(filter) = &self.filter {
                                        match filter {
                                            EventFilter::Pallets(pallets) => {
                                                if pallets
                                                    .contains(&event.pallet_name().to_string())
                                                {
                                                    Some(event)
                                                } else {
                                                    None
                                                }
                                            }
                                            EventFilter::Events(events) => {
                                                if let Some(event_names) =
                                                    events.get(&event.pallet_name().to_string())
                                                {
                                                    if event_names
                                                        .contains(&event.variant_name().to_string())
                                                    {
                                                        Some(event)
                                                    } else {
                                                        None
                                                    }
                                                } else {
                                                    None
                                                }
                                            }
                                        }
                                    } else {
                                        Some(event)
                                    }
                                }
                                Err(e) => {
                                    panic!("event decode from metadata failed for: {e:?}");
                                }
                            })
                            .collect();
                        if let Err(e) = self.handler.send((mode, block, hash, events)).await {
                            panic!("handle_blocks_events(send events to handler err: {e:?})");
                        }
                    }
                    None => {
                        panic!("handle_blocks_events(get empty block hash by number: {block:?})");
                    }
                },
                Err(e) => {
                    panic!("handle_blocks_events(get block hash by number: {block:?} failed for: {e:?})");
                }
            }
        }
    }
}

pub async fn get_events(
    client: &SubClient,
    block: u32,
    pallets: Option<Vec<&str>>,
) -> anyhow::Result<(Hash, Vec<EventDetails<NodeConfig>>)> {
    let hash = block_hash(
        client,
        block,
        None,
    ).await.map_err(|e| anyhow::anyhow!("{e:?}"))?.ok_or(anyhow::anyhow!("no block hash for block {block}"))?;
    let events = match client.client.read().await.events().at(hash).await {
        Ok(events) => events,
        Err(e) => anyhow::bail!("get events for block: {block}, hash: {hash:?} failed for: {e:?}"),
    };
    let mut event_details = vec![];
    for event in events.iter().into_iter() {
        match event {
            Ok(event) => {
                if let Some(pallets) = &pallets {
                    if pallets.contains(&event.pallet_name()) {
                        event_details.push(event);
                    }
                } else {
                    event_details.push(event);
                }
            }
            Err(e) => anyhow::bail!("parse event failed for {e:?}"),
        }
    }
    Ok((hash, event_details))
}

pub async fn get_block_hash(
    client: SubClient,
    mode: WatcherMode,
) -> Result<<NodeConfig as Config>::Hash, String> {
    let guard_client = client.client.read().await;
    match mode {
        WatcherMode::Latest => match latest_block_hash(&client, None).await {
            Ok(hash) => return Ok(hash),
            Err(e) => {
                drop(guard_client);
                log::error!("get latest block failed for : {e:?}, try to rebuild client");
                let err_str = e.to_string();
                if let Err(e) = client.handle_error(e).await {
                    return Err(e.to_string());
                }
                return Err(err_str);
            }
        },
        WatcherMode::Finalized => {
            match guard_client.backend().latest_finalized_block_ref().await {
                Ok(hash) => return Ok(hash.hash()),
                Err(e) => {
                    drop(guard_client);
                    log::error!("event watcher get finalized block failed for : {e:?}, try to rebuild client");
                    let err_str = e.to_string();
                    if let Err(e) = client.handle_error(e).await {
                        return Err(e.to_string());
                    }
                    return Err(err_str);
                }
            }
        }
        WatcherMode::Both => {
            Err("function get_block_hash doesn't support mode: WatcherMode::Both".to_string())
        }
    }
}

pub async fn get_block_number(
    client: SubClient,
    hash: Option<<NodeConfig as Config>::Hash>,
) -> Result<u32, String> {
    let guard_client = client.client.read().await;
    let block_api = guard_client.blocks();
    return match block_api.at_or_latest(hash.map(|v| BlockRef::from(v))).await {
        Ok(block) => Ok(block.number()),
        Err(e) => {
            drop(guard_client);
            log::error!("event watcher get block by hash: {hash:?} failed for: {e:?}, try to rebuild client");
            let err_str = e.to_string();
            if let Err(e) = client.handle_error(e).await {
                return Err(e.to_string());
            }
            Err(err_str)
        }
    }
}
