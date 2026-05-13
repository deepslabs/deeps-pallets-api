#![deny(unused_crate_dependencies)]
pub mod client;
pub mod event_watcher;
pub mod monitor_rpc;
pub mod query;
pub mod submit;
pub mod types;
pub mod watcher_rpc;

pub use crate::client::NodeConfig;
pub use deeps_node_primitives;
use deeps_node_primitives::CustomError;
pub use subxt::constants::Address;
pub use subxt::events::StaticEvent;
pub use subxt::tx::{EcdsaSigner, SecretKey};
pub use subxt::{error::RpcError, events::EventDetails, subxt, Error, JsonRpseeError, config::polkadot::PolkadotExtrinsicParamsBuilder};
pub use subxt::ext::subxt_core::utils::AccountId20;

/// use subxt cli to update metadata 'subxt metadata --url http://127.0.0.1:9944 --version 14 -f bytes > metadata.scale'
#[subxt::subxt(
    runtime_metadata_path = "./metadata.scale",
    derive_for_all_types = "Eq, PartialEq, Clone, Debug"
)]
pub mod deepsafe {}

pub type DeepSafeSubClient = client::SubClient<NodeConfig, EcdsaSigner<NodeConfig>>;

#[derive(Debug, PartialEq)]
pub enum CommitteeEvent {
    CreateCommittee,
    CommitteeCreateFinished,
    ApplyEpochChange,
    BindAnchor,
    CommitteeStartWork,
    StopCommittee,
    UpdateConfigs,
    KeyGenerate,
    KeyHandover,
    ExposeIdentity,
    Unknown,
}

impl CommitteeEvent {
    pub fn event_names() -> Vec<String> {
        vec![
            "CreateCommittee".into(),
            "CommitteeCreateFinished".into(),
            "ApplyEpochChange".into(),
            "BindAnchor".into(),
            "CommitteeStartWork".into(),
            "StopCommittee".into(),
            "UpdateConfigs".into(),
            "KeyGenerate".into(),
            "KeyHandover".into(),
            "ExposeIdentity".into(),
        ]
    }
}

#[derive(Debug, PartialEq)]
pub enum CommitteeHealthEvent {
    Challenges,
    HealthReport,
    ConfirmDHCState,
    PunishEvilDevice,
    Unknown,
}

impl CommitteeHealthEvent {
    pub fn event_names() -> Vec<String> {
        vec![
            "Challenges".into(),
            "HealthReport".into(),
            "ConfirmDHCState".into(),
            "PunishEvilDevice".into(),
        ]
    }
}

#[derive(Debug, PartialEq)]
pub enum ConfigsEvent {
    ConfigUpdate,
    Unknown,
}

impl ConfigsEvent {
    pub fn event_names() -> Vec<String> {
        vec!["ConfigUpdate".into()]
    }
}

#[derive(Debug, PartialEq)]
pub enum RpcEvent {
    DeviceRegistered,
    Unknown,
}

impl RpcEvent {
    pub fn event_names() -> Vec<String> {
        vec!["DeviceRegistered".into()]
    }
}

#[derive(Debug, PartialEq)]
pub enum ChannelEvent {
    NewTransaction,
    SubmitTransactionSignResult,
    Connection,
    NewSourceHash,
    RefreshInscription,
    SignRefresh,
    SubmitRefresh,
    RequestNewIssueXudt,
    SignIssueXudt,
    SignIssueXudtFinished,
    UpdateIssueXudtStatus,
    SignNewUid,
    SubmitSignNewUidResult,
    UpdateChannelMappingTick,
    UpdateCommitteeFeeConfig,
    RequestForcedWithdrawal,
    SignForcedWithdrawal,
    FinishForcedWithdrawal,
    Unknown,
}

impl ChannelEvent {
    pub fn event_names() -> Vec<String> {
        vec![
            "NewTransaction".into(),
            "SubmitTransactionSignResult".into(),
            "Connection".into(),
            "NewSourceHash".into(),
            "RefreshInscription".into(),
            "SignRefresh".into(),
            "SubmitRefresh".into(),
            "RequestNewIssueXudt".into(),
            "SignIssueXudt".into(),
            "SignIssueXudtFinished".into(),
            "UpdateIssueXudtStatus".into(),
            "SignNewUid".into(),
            "SubmitSignNewUidResult".into(),
            "UpdateChannelMappingTick".into(),
            "UpdateCommitteeFeeConfig".into(),
            "RequestForcedWithdrawal".into(),
            "SignForcedWithdrawal".into(),
            "FinishForcedWithdrawal".into(),
        ]
    }
}

#[derive(Debug, PartialEq)]
pub enum MiningEvent {
    NewChallenge,
    Heartbeat,
    DeviceRegistered,
    DeviceJoinService,
    DeviceTryExitService,
    DeviceExitService,
    DeviceRemoved,
    Unknown,
}

impl MiningEvent {
    pub fn event_names() -> Vec<String> {
        vec![
            "NewChallenge".into(),
            "Heartbeat".into(),
            "DeviceRegistered".into(),
            "DeviceJoinService".into(),
            "DeviceTryExitService".into(),
            "DeviceExitService".into(),
            "DeviceRemoved".into(),
        ]
    }
}

#[derive(Debug, PartialEq)]
pub enum CommitteeAssetsEvent {
    RefreshAssets,
    Unknown,
}

impl CommitteeAssetsEvent {
    pub fn event_names() -> Vec<String> {
        vec!["RefreshAssets".into()]
    }
}

impl std::str::FromStr for CommitteeEvent {
    type Err = ();
    fn from_str(input: &str) -> Result<CommitteeEvent, Self::Err> {
        match input {
            "CreateCommittee" => Ok(CommitteeEvent::CreateCommittee),
            "CommitteeCreateFinished" => Ok(CommitteeEvent::CommitteeCreateFinished),
            "ApplyEpochChange" => Ok(CommitteeEvent::ApplyEpochChange),
            "BindAnchor" => Ok(CommitteeEvent::BindAnchor),
            "CommitteeStartWork" => Ok(CommitteeEvent::CommitteeStartWork),
            "StopCommittee" => Ok(CommitteeEvent::StopCommittee),
            "UpdateConfigs" => Ok(CommitteeEvent::UpdateConfigs),
            "KeyGenerate" => Ok(CommitteeEvent::KeyGenerate),
            "KeyHandover" => Ok(CommitteeEvent::KeyHandover),
            "ExposeIdentity" => Ok(CommitteeEvent::ExposeIdentity),
            _ => Ok(CommitteeEvent::Unknown),
        }
    }
}

impl std::str::FromStr for CommitteeAssetsEvent {
    type Err = ();
    fn from_str(input: &str) -> Result<CommitteeAssetsEvent, Self::Err> {
        match input {
            "RefreshAssets" => Ok(CommitteeAssetsEvent::RefreshAssets),
            _ => Ok(CommitteeAssetsEvent::Unknown),
        }
    }
}

impl std::str::FromStr for CommitteeHealthEvent {
    type Err = ();
    fn from_str(input: &str) -> Result<CommitteeHealthEvent, Self::Err> {
        match input {
            "Challenges" => Ok(CommitteeHealthEvent::Challenges),
            "HealthReport" => Ok(CommitteeHealthEvent::HealthReport),
            "ConfirmDHCState" => Ok(CommitteeHealthEvent::ConfirmDHCState),
            "PunishEvilDevice" => Ok(CommitteeHealthEvent::PunishEvilDevice),
            _ => Ok(CommitteeHealthEvent::Unknown),
        }
    }
}

impl std::str::FromStr for ConfigsEvent {
    type Err = ();
    fn from_str(input: &str) -> Result<ConfigsEvent, Self::Err> {
        match input {
            "ConfigUpdate" => Ok(ConfigsEvent::ConfigUpdate),
            _ => Ok(ConfigsEvent::Unknown),
        }
    }
}

impl std::str::FromStr for ChannelEvent {
    type Err = ();
    fn from_str(input: &str) -> Result<ChannelEvent, Self::Err> {
        match input {
            "NewTransaction" => Ok(ChannelEvent::NewTransaction),
            "SubmitTransactionSignResult" => Ok(ChannelEvent::SubmitTransactionSignResult),
            "Connection" => Ok(ChannelEvent::Connection),
            "NewSourceHash" => Ok(ChannelEvent::NewSourceHash),
            "RefreshInscription" => Ok(ChannelEvent::RefreshInscription),
            "SignRefresh" => Ok(ChannelEvent::SignRefresh),
            "SubmitRefresh" => Ok(ChannelEvent::SubmitRefresh),
            "RequestNewIssueXudt" => Ok(ChannelEvent::RequestNewIssueXudt),
            "SignIssueXudt" => Ok(ChannelEvent::SignIssueXudt),
            "SignIssueXudtFinished" => Ok(ChannelEvent::SignIssueXudtFinished),
            "UpdateIssueXudtStatus" => Ok(ChannelEvent::UpdateIssueXudtStatus),
            "UpdateChannelMappingTick" => Ok(ChannelEvent::UpdateChannelMappingTick),
            "UpdateCommitteeFeeConfig" => Ok(ChannelEvent::UpdateCommitteeFeeConfig),
            "SignNewUid" => Ok(ChannelEvent::SignNewUid),
            "SubmitSignNewUidResult" => Ok(ChannelEvent::SubmitSignNewUidResult),
            "RequestForcedWithdrawal" => Ok(ChannelEvent::RequestForcedWithdrawal),
            "SignForcedWithdrawal" => Ok(ChannelEvent::SignForcedWithdrawal),
            "FinishForcedWithdrawal" => Ok(ChannelEvent::FinishForcedWithdrawal),
            _ => Ok(ChannelEvent::Unknown),
        }
    }
}

impl std::str::FromStr for MiningEvent {
    type Err = ();
    fn from_str(input: &str) -> Result<MiningEvent, Self::Err> {
        match input {
            "NewChallenge" => Ok(MiningEvent::NewChallenge),
            "Heartbeat" => Ok(MiningEvent::Heartbeat),
            "DeviceRegistered" => Ok(MiningEvent::DeviceRegistered),
            "DeviceJoinService" => Ok(MiningEvent::DeviceJoinService),
            "DeviceTryExitService" => Ok(MiningEvent::DeviceTryExitService),
            "DeviceExitService" => Ok(MiningEvent::DeviceExitService),
            "DeviceRemoved" => Ok(MiningEvent::DeviceRemoved),
            _ => Ok(MiningEvent::Unknown),
        }
    }
}

impl std::str::FromStr for RpcEvent {
    type Err = ();
    fn from_str(input: &str) -> Result<RpcEvent, Self::Err> {
        match input {
            "DeviceRegistered" => Ok(RpcEvent::DeviceRegistered),
            _ => Ok(RpcEvent::Unknown),
        }
    }
}

pub(crate) fn convert_to_custom_error(custom: u8) -> String {
    let err = CustomError::from_num(custom);
    err.to_string()
}

pub(crate) fn handle_custom_error(error: Error) -> String {
    if let Error::Rpc(RpcError::ClientError(e)) = error {
        let err = e.to_string();
        parse_custom_err_from_string_err(err)
    } else {
        error.to_string()
    }
}

fn parse_custom_err_from_string_err(err: String) -> String {
    // only try to extract 'custom number', will return input if parse error
    let v: Vec<&str> = err.split("Custom error: ").collect();
    if v.len() == 2 {
        let vv: Vec<&str> = v[1].split('\"').collect();
        if vv.len() == 2 {
            if let Ok(num) = vv[0].parse::<u8>() {
                return convert_to_custom_error(num);
            }
        }
    }
    err
}

pub fn no_prefix<T: AsRef<str>>(data: T) -> String {
    data.as_ref()
        .strip_prefix("0x")
        .unwrap_or(data.as_ref())
        .to_string()
}
