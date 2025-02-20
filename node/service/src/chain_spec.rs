#[cfg(feature = "cere-dev-native")]
use cere_dev_runtime as cere_dev;
#[cfg(feature = "cere-native")]
use cere_runtime as cere;
#[cfg(feature = "cere-dev-native")]
use cere_runtime_common::constants::currency::DOLLARS as TEST_UNITS;
use ddc_primitives::sr25519::AuthorityId as DdcVerificationId;
use jsonrpsee::core::__reexports::serde_json;
pub use node_primitives::{AccountId, Balance, Block, Signature};
use pallet_im_online::sr25519::AuthorityId as ImOnlineId;
use sc_chain_spec::ChainSpecExtension;
use sc_service::ChainType;
use serde::{Deserialize, Serialize};
use sp_authority_discovery::AuthorityId as AuthorityDiscoveryId;
use sp_consensus_babe::AuthorityId as BabeId;
use sp_consensus_grandpa::AuthorityId as GrandpaId;
use sp_core::{sr25519, Pair, Public};
use sp_runtime::{
	traits::{IdentifyAccount, Verify},
	Perbill,
};

const DEFAULT_PROTOCOL_ID: &str = "cere";

#[derive(Default, Clone, Serialize, Deserialize, ChainSpecExtension)]
#[serde(rename_all = "camelCase")]
pub struct Extensions {
	/// Block numbers with known hashes.
	pub fork_blocks: sc_client_api::ForkBlocks<Block>,
	/// Known bad block hashes.
	pub bad_blocks: sc_client_api::BadBlocks<Block>,
	/// The light sync state extension used by the sync-state rpc.
	pub light_sync_state: sc_sync_state_rpc::LightSyncStateExtension,
}

// Dummy chain spec, in case when we don't have the native runtime.
pub type DummyChainSpec = sc_service::GenericChainSpec<(), Extensions>;

/// The `ChainSpec` parameterized for the cere runtime.
#[cfg(feature = "cere-native")]
pub type CereChainSpec = sc_service::GenericChainSpec<cere::RuntimeGenesisConfig, Extensions>;

/// The `ChainSpec` parameterized for the cere runtime.
// Dummy chain spec, but that is fine when we don't have the native runtime.
#[cfg(not(feature = "cere-native"))]
pub type CereChainSpec = DummyChainSpec;

/// The `ChainSpec` parameterized for the cere-dev runtime.
#[cfg(feature = "cere-dev-native")]
pub type CereDevChainSpec =
	sc_service::GenericChainSpec<cere_dev::RuntimeGenesisConfig, Extensions>;

/// The `ChainSpec` parameterized for the cere-dev runtime.
// Dummy chain spec, but that is fine when we don't have the native runtime.
#[cfg(not(feature = "cere-dev-native"))]
pub type CereDevChainSpec = DummyChainSpec;

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
	TPublic::Pair::from_string(&format!("//{}", seed), None)
		.expect("static values are valid; qed")
		.public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
	AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
	AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

// Helper function to generate stash, controller and session key from seed
pub fn authority_keys_from_seed(
	seed: &str,
) -> (AccountId, AccountId, GrandpaId, BabeId, ImOnlineId, AuthorityDiscoveryId, DdcVerificationId)
{
	(
		get_account_id_from_seed::<sr25519::Public>(&format!("{}//stash", seed)),
		get_account_id_from_seed::<sr25519::Public>(seed),
		get_from_seed::<GrandpaId>(seed),
		get_from_seed::<BabeId>(seed),
		get_from_seed::<ImOnlineId>(seed),
		get_from_seed::<AuthorityDiscoveryId>(seed),
		get_from_seed::<DdcVerificationId>(seed),
	)
}

#[cfg(feature = "cere-dev-native")]
fn cere_dev_session_keys(
	grandpa: GrandpaId,
	babe: BabeId,
	im_online: ImOnlineId,
	authority_discovery: AuthorityDiscoveryId,
	ddc_verification: DdcVerificationId,
) -> cere_dev::SessionKeys {
	cere_dev::SessionKeys { grandpa, babe, im_online, authority_discovery, ddc_verification }
}

/// Helper function to create Cere Dev `RuntimeGenesisConfig` for testing
#[cfg(feature = "cere-dev-native")]
pub fn cere_dev_genesis(
	initial_authorities: Vec<(
		AccountId,
		AccountId,
		GrandpaId,
		BabeId,
		ImOnlineId,
		AuthorityDiscoveryId,
		DdcVerificationId,
	)>,
	initial_nominators: Vec<AccountId>,
	root_key: AccountId,
	endowed_accounts: Option<Vec<AccountId>>,
) -> cere_dev::RuntimeGenesisConfig {
	let mut endowed_accounts: Vec<AccountId> = endowed_accounts.unwrap_or_else(|| {
		vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Charlie"),
			get_account_id_from_seed::<sr25519::Public>("Dave"),
			get_account_id_from_seed::<sr25519::Public>("Eve"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie"),
			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
			get_account_id_from_seed::<sr25519::Public>("Charlie//stash"),
			get_account_id_from_seed::<sr25519::Public>("Dave//stash"),
			get_account_id_from_seed::<sr25519::Public>("Eve//stash"),
			get_account_id_from_seed::<sr25519::Public>("Ferdie//stash"),
		]
	});

	// endow all authorities and nominators.
	initial_authorities
		.iter()
		.map(|x| &x.0)
		.chain(initial_nominators.iter())
		.for_each(|x| {
			if !endowed_accounts.contains(x) {
				endowed_accounts.push(x.clone())
			}
		});

	// stakers: all validators and nominators.
	let mut rng = rand::thread_rng();
	let stakers = initial_authorities
		.iter()
		.map(|x| (x.0.clone(), x.1.clone(), STASH, cere_dev::StakerStatus::Validator))
		.chain(initial_nominators.iter().map(|x| {
			use rand::{seq::SliceRandom, Rng};
			let limit = (cere_dev::MaxNominations::get() as usize).min(initial_authorities.len());
			let count = rng.gen::<usize>() % limit;
			let nominations = initial_authorities
				.as_slice()
				.choose_multiple(&mut rng, count)
				.map(|choice| choice.0.clone())
				.collect::<Vec<_>>();
			(x.clone(), x.clone(), STASH, cere_dev::StakerStatus::Nominator(nominations))
		}))
		.collect::<Vec<_>>();

	const ENDOWMENT: Balance = 10_000_000_000 * TEST_UNITS;
	const STASH: Balance = ENDOWMENT / 1000;

	cere_dev::RuntimeGenesisConfig {
		system: cere_dev::SystemConfig::default(),
		balances: cere_dev::BalancesConfig {
			// Configure endowed accounts with initial balance of 1 << 60.
			balances: endowed_accounts.iter().cloned().map(|x| (x, ENDOWMENT)).collect(),
		},
		indices: cere_dev::IndicesConfig { indices: vec![] },
		session: cere_dev::SessionConfig {
			keys: initial_authorities
				.iter()
				.map(|x| {
					(
						x.0.clone(),
						x.0.clone(),
						cere_dev_session_keys(
							x.2.clone(),
							x.3.clone(),
							x.4.clone(),
							x.5.clone(),
							x.6.clone(),
						),
					)
				})
				.collect::<Vec<_>>(),
		},
		staking: cere_dev::StakingConfig {
			validator_count: initial_authorities.len() as u32,
			minimum_validator_count: initial_authorities.len() as u32,
			invulnerables: initial_authorities.iter().map(|x| x.0.clone()).collect(),
			slash_reward_fraction: Perbill::from_percent(10),
			stakers,
			..Default::default()
		},
		ddc_staking: cere_dev::DdcStakingConfig::default(),
		sudo: cere_dev::SudoConfig { key: Some(root_key) },
		babe: cere_dev::BabeConfig {
			authorities: Default::default(),
			epoch_config: Some(cere_dev::BABE_GENESIS_EPOCH_CONFIG),
			..Default::default()
		},
		im_online: cere_dev::ImOnlineConfig { keys: vec![] },
		authority_discovery: cere_dev::AuthorityDiscoveryConfig {
			keys: vec![],
			..Default::default()
		},
		grandpa: Default::default(),
		treasury: Default::default(),
		vesting: Default::default(),
		transaction_payment: Default::default(),
		ddc_customers: Default::default(),
		nomination_pools: Default::default(),
		ddc_clusters: Default::default(),
		ddc_nodes: Default::default(),
		ddc_payouts: Default::default(),
		tech_comm: cere_dev::TechCommConfig {
			members: endowed_accounts
				.iter()
				.take((endowed_accounts.len() + 1) / 2)
				.cloned()
				.collect(),
			phantom: Default::default(),
		},
		ddc_verification: Default::default(),
	}
}

/// Returns the properties for the [`cere-dev-native`].
pub fn cere_dev_native_chain_spec_properties() -> serde_json::map::Map<String, serde_json::Value> {
	serde_json::json!({
		"tokenDecimals": 10,
		"tokenSymbol": "CERE",
		"ss58Format": 54,
	})
	.as_object()
	.expect("Map given; qed")
	.clone()
}

/// Helper function to create Cere `RuntimeGenesisConfig` for testing
#[cfg(feature = "cere-dev-native")]
fn cere_dev_config_genesis() -> cere_dev::RuntimeGenesisConfig {
	cere_dev_genesis(
		// Initial authorities
		vec![authority_keys_from_seed("Alice")],
		// Initial nominators
		vec![],
		// Sudo account
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		// Pre-funded accounts
		Some(vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
		]),
	)
}

#[cfg(feature = "cere-dev-native")]
pub fn cere_dev_development_config() -> Result<CereDevChainSpec, String> {
	let wasm_binary = cere_dev::WASM_BINARY.ok_or("Cere Dev development wasm not available")?;

	#[allow(deprecated)]
	Ok(CereDevChainSpec::from_genesis(
		"Development",
		"cere_dev",
		ChainType::Development,
		cere_dev_config_genesis,
		vec![],
		None,
		Some(DEFAULT_PROTOCOL_ID),
		None,
		Some(cere_dev_native_chain_spec_properties()),
		Default::default(),
		wasm_binary,
	))
}

#[cfg(feature = "cere-dev-native")]
fn cere_dev_local_testnet_genesis() -> cere_dev::RuntimeGenesisConfig {
	cere_dev_genesis(
		// Initial authorities
		vec![
			authority_keys_from_seed("Alice"),
			authority_keys_from_seed("Bob"),
			authority_keys_from_seed("Charlie"),
		],
		// Initial nominators
		vec![],
		// Sudo account
		get_account_id_from_seed::<sr25519::Public>("Alice"),
		// Pre-funded accounts
		Some(vec![
			get_account_id_from_seed::<sr25519::Public>("Alice"),
			get_account_id_from_seed::<sr25519::Public>("Bob"),
			get_account_id_from_seed::<sr25519::Public>("Alice//stash"),
			get_account_id_from_seed::<sr25519::Public>("Bob//stash"),
		]),
	)
}

#[cfg(feature = "cere-dev-native")]
pub fn cere_dev_local_testnet_config() -> Result<CereDevChainSpec, String> {
	let wasm_binary = cere_dev::WASM_BINARY.ok_or("Cere Dev development wasm not available")?;

	#[allow(deprecated)]
	Ok(CereDevChainSpec::from_genesis(
		"Local Testnet",
		"cere_dev_local_testnet",
		ChainType::Local,
		cere_dev_local_testnet_genesis,
		vec![],
		None,
		Some(DEFAULT_PROTOCOL_ID),
		None,
		None,
		Default::default(),
		wasm_binary,
	))
}

pub fn cere_mainnet_config() -> Result<CereChainSpec, String> {
	CereChainSpec::from_json_bytes(&include_bytes!("../chain-specs/mainnet.json")[..])
}

pub fn cere_testnet_config() -> Result<CereChainSpec, String> {
	CereChainSpec::from_json_bytes(&include_bytes!("../chain-specs/testnet.json")[..])
}

pub fn cere_qanet_config() -> Result<CereChainSpec, String> {
	CereChainSpec::from_json_bytes(&include_bytes!("../chain-specs/qanet.json")[..])
}

pub fn cere_devnet_config() -> Result<CereDevChainSpec, String> {
	CereDevChainSpec::from_json_bytes(&include_bytes!("../chain-specs/devnet.json")[..])
}
