//! Service and ServiceFactory implementation. Specialized wrapper over substrate service.

use std::sync::Arc;

#[cfg(not(any(feature = "cere-native", feature = "cere-dev-native",)))]
compile_error!("at least one runtime feature must be enabled");

#[cfg(feature = "cere-dev-native")]
pub use cere_dev_runtime;
#[cfg(feature = "cere-native")]
pub use cere_runtime;
use futures::prelude::*;
use sc_client_api::{Backend, BlockBackend};
use sc_consensus_babe::SlotProportion;
pub use sc_executor::NativeExecutionDispatch;
use sc_network::{Event, NetworkEventStream};
use sc_service::{
	error::Error as ServiceError, Configuration, KeystoreContainer, RpcHandlers, TaskManager,
};
use sc_telemetry::{Telemetry, TelemetryWorker};
use sp_runtime::traits::{BlakeTwo256, Block as BlockT};
pub mod chain_spec;
#[cfg(feature = "cere-dev-native")]
pub use cere_client::CereDevExecutorDispatch;
#[cfg(feature = "cere-native")]
pub use cere_client::CereExecutorDispatch;
pub use cere_client::{
	AbstractClient, Client, ClientHandle, ExecuteWithClient, FullBackend, FullClient,
	RuntimeApiCollection,
};
pub use chain_spec::{CereChainSpec, CereDevChainSpec};
pub use node_primitives::{Block, BlockNumber};
use sc_executor::{
	HeapAllocStrategy, NativeElseWasmExecutor, WasmExecutor, DEFAULT_HEAP_ALLOC_STRATEGY,
};
pub use sc_service::ChainSpec;
use sc_transaction_pool_api::OffchainTransactionPoolFactory;
pub use sp_api::ConstructRuntimeApi;

/// The minimum period of blocks on which justifications will be
/// imported and generated.
const GRANDPA_JUSTIFICATION_PERIOD: u32 = 512;

type FullSelectChain = sc_consensus::LongestChain<FullBackend, Block>;
type FullGrandpaBlockImport<RuntimeApi, ExecutorDispatch> =
	sc_consensus_grandpa::GrandpaBlockImport<
		FullBackend,
		Block,
		FullClient<RuntimeApi, ExecutorDispatch>,
		FullSelectChain,
	>;

struct Basics<RuntimeApi, ExecutorDispatch>
where
	RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi, ExecutorDispatch>>
		+ Send
		+ Sync
		+ 'static,
	RuntimeApi::RuntimeApi: RuntimeApiCollection,
	ExecutorDispatch: NativeExecutionDispatch + 'static,
{
	task_manager: TaskManager,
	client: Arc<FullClient<RuntimeApi, ExecutorDispatch>>,
	backend: Arc<FullBackend>,
	keystore_container: KeystoreContainer,
	telemetry: Option<Telemetry>,
}

fn new_partial_basics<RuntimeApi, ExecutorDispatch>(
	config: &Configuration,
) -> Result<Basics<RuntimeApi, ExecutorDispatch>, ServiceError>
where
	RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi, ExecutorDispatch>>
		+ Send
		+ Sync
		+ 'static,
	RuntimeApi::RuntimeApi: RuntimeApiCollection,
	ExecutorDispatch: NativeExecutionDispatch + 'static,
{
	let telemetry = config
		.telemetry_endpoints
		.clone()
		.filter(|x| !x.is_empty())
		.map(|endpoints| -> Result<_, sc_telemetry::Error> {
			let worker = TelemetryWorker::new(16)?;
			let telemetry = worker.handle().new_telemetry(endpoints);
			Ok((worker, telemetry))
		})
		.transpose()?;

	let heap_pages = config
		.default_heap_pages
		.map_or(DEFAULT_HEAP_ALLOC_STRATEGY, |h| HeapAllocStrategy::Static { extra_pages: h as _ });

	let wasm = WasmExecutor::builder()
		.with_execution_method(config.wasm_method)
		.with_onchain_heap_alloc_strategy(heap_pages)
		.with_offchain_heap_alloc_strategy(heap_pages)
		.with_max_runtime_instances(config.max_runtime_instances)
		.with_runtime_cache_size(config.runtime_cache_size)
		.build();

	let executor = NativeElseWasmExecutor::<ExecutorDispatch>::new_with_wasm_executor(wasm);

	let (client, backend, keystore_container, task_manager) =
		sc_service::new_full_parts::<Block, RuntimeApi, _>(
			config,
			telemetry.as_ref().map(|(_, telemetry)| telemetry.handle()),
			executor,
		)?;

	let client = Arc::new(client);

	let telemetry = telemetry.map(|(worker, telemetry)| {
		task_manager.spawn_handle().spawn("telemetry", None, worker.run());
		telemetry
	});

	Ok(Basics { task_manager, client, backend, keystore_container, telemetry })
}

#[allow(clippy::type_complexity)]
fn new_partial<RuntimeApi, ExecutorDispatch>(
	config: &Configuration,
	Basics { task_manager, backend, client, keystore_container, telemetry }: Basics<
		RuntimeApi,
		ExecutorDispatch,
	>,
) -> Result<
	sc_service::PartialComponents<
		FullClient<RuntimeApi, ExecutorDispatch>,
		FullBackend,
		FullSelectChain,
		sc_consensus::DefaultImportQueue<Block>,
		sc_transaction_pool::FullPool<Block, FullClient<RuntimeApi, ExecutorDispatch>>,
		(
			impl Fn(
				cere_rpc::DenyUnsafe,
				sc_rpc::SubscriptionTaskExecutor,
			) -> Result<jsonrpsee::RpcModule<()>, sc_service::Error>,
			(
				sc_consensus_babe::BabeBlockImport<
					Block,
					FullClient<RuntimeApi, ExecutorDispatch>,
					FullGrandpaBlockImport<RuntimeApi, ExecutorDispatch>,
				>,
				sc_consensus_grandpa::LinkHalf<
					Block,
					FullClient<RuntimeApi, ExecutorDispatch>,
					FullSelectChain,
				>,
				sc_consensus_babe::BabeLink<Block>,
			),
			sc_consensus_grandpa::SharedVoterState,
			Option<Telemetry>,
		),
	>,
	ServiceError,
>
where
	RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi, ExecutorDispatch>>
		+ Send
		+ Sync
		+ 'static,
	RuntimeApi::RuntimeApi: RuntimeApiCollection,
	ExecutorDispatch: NativeExecutionDispatch + 'static,
{
	let select_chain = sc_consensus::LongestChain::new(backend.clone());

	let transaction_pool = sc_transaction_pool::BasicPool::new_full(
		config.transaction_pool.clone(),
		config.role.is_authority().into(),
		config.prometheus_registry(),
		task_manager.spawn_essential_handle(),
		client.clone(),
	);
	let (grandpa_block_import, grandpa_link) = sc_consensus_grandpa::block_import(
		client.clone(),
		GRANDPA_JUSTIFICATION_PERIOD,
		&client,
		select_chain.clone(),
		telemetry.as_ref().map(|x| x.handle()),
	)?;

	let justification_import = grandpa_block_import.clone();

	let (block_import, babe_link) = sc_consensus_babe::block_import(
		sc_consensus_babe::configuration(&*client)?,
		grandpa_block_import,
		client.clone(),
	)?;

	let slot_duration = babe_link.config().slot_duration();
	let (import_queue, babe_worker_handle) =
		sc_consensus_babe::import_queue(sc_consensus_babe::ImportQueueParams {
			link: babe_link.clone(),
			block_import: block_import.clone(),
			justification_import: Some(Box::new(justification_import)),
			client: client.clone(),
			select_chain: select_chain.clone(),
			create_inherent_data_providers: move |_, ()| async move {
				let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

				let slot =
				sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
					*timestamp,
					slot_duration,
				);

				Ok((slot, timestamp))
			},
			spawner: &task_manager.spawn_essential_handle(),
			registry: config.prometheus_registry(),
			telemetry: telemetry.as_ref().map(|x| x.handle()),
			offchain_tx_pool_factory: OffchainTransactionPoolFactory::new(transaction_pool.clone()),
		})?;

	let import_setup = (block_import, grandpa_link, babe_link);

	let (rpc_extensions_builder, rpc_setup) = {
		let (_, grandpa_link, _babe_link) = &import_setup;

		let justification_stream = grandpa_link.justification_stream();
		let shared_authority_set = grandpa_link.shared_authority_set().clone();
		let shared_voter_state = sc_consensus_grandpa::SharedVoterState::empty();
		let shared_voter_state2 = shared_voter_state.clone();

		let finality_proof_provider = sc_consensus_grandpa::FinalityProofProvider::new_for_service(
			backend.clone(),
			Some(shared_authority_set.clone()),
		);

		let client = client.clone();
		let pool = transaction_pool.clone();
		let select_chain = select_chain.clone();
		let keystore = keystore_container.keystore();
		let chain_spec = config.chain_spec.cloned_box();

		let rpc_backend = backend.clone();
		let rpc_extensions_builder = move |deny_unsafe, subscription_executor| {
			let deps = cere_rpc::FullDeps {
				client: client.clone(),
				pool: pool.clone(),
				select_chain: select_chain.clone(),
				chain_spec: chain_spec.cloned_box(),
				deny_unsafe,
				babe: cere_rpc::BabeDeps {
					babe_worker_handle: babe_worker_handle.clone(),
					keystore: keystore.clone(),
				},
				grandpa: cere_rpc::GrandpaDeps {
					shared_voter_state: shared_voter_state.clone(),
					shared_authority_set: shared_authority_set.clone(),
					justification_stream: justification_stream.clone(),
					subscription_executor,
					finality_provider: finality_proof_provider.clone(),
				},
			};

			cere_rpc::create_full(deps, rpc_backend.clone()).map_err(Into::into)
		};

		(rpc_extensions_builder, shared_voter_state2)
	};

	Ok(sc_service::PartialComponents {
		client,
		backend,
		task_manager,
		keystore_container,
		select_chain,
		import_queue,
		transaction_pool,
		other: (rpc_extensions_builder, import_setup, rpc_setup, telemetry),
	})
}

pub fn build_full(
	config: Configuration,
	disable_hardware_benchmarks: bool,
) -> Result<NewFull<Client>, ServiceError> {
	#[cfg(feature = "cere-dev-native")]
	if config.chain_spec.is_cere_dev() {
		return new_full::<cere_dev_runtime::RuntimeApi, CereDevExecutorDispatch>(
			config,
			disable_hardware_benchmarks,
			|_, _| (),
		)
		.map(|full| full.with_client(Client::CereDev));
	}

	#[cfg(feature = "cere-native")]
	{
		new_full::<cere_runtime::RuntimeApi, CereExecutorDispatch>(
			config,
			disable_hardware_benchmarks,
			|_, _| (),
		)
		.map(|full| full.with_client(Client::Cere))
	}

	#[cfg(not(feature = "cere-native"))]
	Err(ServiceError::NoRuntime)
}

pub struct NewFull<C> {
	pub task_manager: TaskManager,
	pub client: C,
	pub network: Arc<sc_network::NetworkService<Block, <Block as BlockT>::Hash>>,
	pub rpc_handlers: RpcHandlers,
	pub backend: Arc<FullBackend>,
}

impl<C> NewFull<C> {
	/// Convert the client type using the given `func`.
	pub fn with_client<NC>(self, func: impl FnOnce(C) -> NC) -> NewFull<NC> {
		NewFull {
			task_manager: self.task_manager,
			client: func(self.client),
			network: self.network,
			rpc_handlers: self.rpc_handlers,
			backend: self.backend,
		}
	}
}

pub fn new_full<RuntimeApi, ExecutorDispatch>(
	config: Configuration,
	disable_hardware_benchmarks: bool,
	with_startup_data: impl FnOnce(
		&sc_consensus_babe::BabeBlockImport<
			Block,
			FullClient<RuntimeApi, ExecutorDispatch>,
			FullGrandpaBlockImport<RuntimeApi, ExecutorDispatch>,
		>,
		&sc_consensus_babe::BabeLink<Block>,
	),
) -> Result<NewFull<Arc<FullClient<RuntimeApi, ExecutorDispatch>>>, ServiceError>
where
	RuntimeApi: ConstructRuntimeApi<Block, FullClient<RuntimeApi, ExecutorDispatch>>
		+ Send
		+ Sync
		+ 'static,
	RuntimeApi::RuntimeApi: RuntimeApiCollection,
	ExecutorDispatch: NativeExecutionDispatch + 'static,
{
	use sc_service::WarpSyncParams;

	let hwbench = if !disable_hardware_benchmarks {
		config.database.path().map(|database_path| {
			let _ = std::fs::create_dir_all(database_path);
			sc_sysinfo::gather_hwbench(Some(database_path))
		})
	} else {
		None
	};

	let basics = new_partial_basics::<RuntimeApi, ExecutorDispatch>(&config)?;

	let sc_service::PartialComponents {
		client,
		backend,
		mut task_manager,
		import_queue,
		keystore_container,
		select_chain,
		transaction_pool,
		other: (rpc_builder, import_setup, rpc_setup, mut telemetry),
	} = new_partial(&config, basics)?;

	let shared_voter_state = rpc_setup;
	let auth_disc_publish_non_global_ips = config.network.allow_non_globals_in_dht;

	let mut net_config = sc_network::config::FullNetworkConfiguration::new(&config.network);

	let grandpa_protocol_name = sc_consensus_grandpa::protocol_standard_name(
		&client.block_hash(0).ok().flatten().expect("Genesis block exists; qed"),
		&config.chain_spec,
	);
	net_config.add_notification_protocol(sc_consensus_grandpa::grandpa_peers_set_config(
		grandpa_protocol_name.clone(),
	));

	let warp_sync = Arc::new(sc_consensus_grandpa::warp_proof::NetworkProvider::new(
		backend.clone(),
		import_setup.1.shared_authority_set().clone(),
		Vec::default(),
	));

	let (network, system_rpc_tx, tx_handler_controller, network_starter, sync_service) =
		sc_service::build_network(sc_service::BuildNetworkParams {
			config: &config,
			net_config,
			client: client.clone(),
			transaction_pool: transaction_pool.clone(),
			spawn_handle: task_manager.spawn_handle(),
			import_queue,
			block_announce_validator_builder: None,
			warp_sync_params: Some(WarpSyncParams::WithProvider(warp_sync)),
			block_relay: None,
		})?;

	if config.offchain_worker.enabled {
		task_manager.spawn_handle().spawn(
			"offchain-workers-runner",
			"offchain-worker",
			sc_offchain::OffchainWorkers::new(sc_offchain::OffchainWorkerOptions {
				runtime_api_provider: client.clone(),
				is_validator: config.role.is_authority(),
				keystore: Some(keystore_container.keystore()),
				offchain_db: backend.offchain_storage(),
				transaction_pool: Some(OffchainTransactionPoolFactory::new(
					transaction_pool.clone(),
				)),
				network_provider: network.clone(),
				enable_http_requests: true,
				custom_extensions: |_| vec![],
			})
			.run(client.clone(), task_manager.spawn_handle())
			.boxed(),
		);
	}

	let role = config.role.clone();
	let force_authoring = config.force_authoring;
	let backoff_authoring_blocks =
		Some(sc_consensus_slots::BackoffAuthoringOnFinalizedHeadLagging::default());
	let name = config.network.node_name.clone();
	let enable_grandpa = !config.disable_grandpa;
	let prometheus_registry = config.prometheus_registry().cloned();

	let rpc_handlers = sc_service::spawn_tasks(sc_service::SpawnTasksParams {
		config,
		backend: backend.clone(),
		client: client.clone(),
		keystore: keystore_container.keystore(),
		network: network.clone(),
		rpc_builder: Box::new(rpc_builder),
		transaction_pool: transaction_pool.clone(),
		task_manager: &mut task_manager,
		system_rpc_tx,
		tx_handler_controller,
		telemetry: telemetry.as_mut(),
		sync_service: sync_service.clone(),
	})?;

	if let Some(hwbench) = hwbench {
		sc_sysinfo::print_hwbench(&hwbench);

		if let Some(ref mut telemetry) = telemetry {
			let telemetry_handle = telemetry.handle();
			task_manager.spawn_handle().spawn(
				"telemetry_hwbench",
				None,
				sc_sysinfo::initialize_hwbench_telemetry(telemetry_handle, hwbench),
			);
		}
	}

	let (block_import, grandpa_link, babe_link) = import_setup;

	(with_startup_data)(&block_import, &babe_link);

	if let sc_service::config::Role::Authority { .. } = &role {
		let proposer = sc_basic_authorship::ProposerFactory::new(
			task_manager.spawn_handle(),
			client.clone(),
			transaction_pool.clone(),
			prometheus_registry.as_ref(),
			telemetry.as_ref().map(|x| x.handle()),
		);

		let client_clone = client.clone();
		let slot_duration = babe_link.config().slot_duration();
		let babe_config = sc_consensus_babe::BabeParams {
			keystore: keystore_container.keystore(),
			client: client.clone(),
			select_chain,
			env: proposer,
			block_import,
			sync_oracle: sync_service.clone(),
			justification_sync_link: sync_service.clone(),
			create_inherent_data_providers: move |parent, ()| {
				let client_clone = client_clone.clone();
				async move {
					let timestamp = sp_timestamp::InherentDataProvider::from_system_time();

					let slot =
						sp_consensus_babe::inherents::InherentDataProvider::from_timestamp_and_slot_duration(
							*timestamp,
							slot_duration,
						);

					let storage_proof =
						sp_transaction_storage_proof::registration::new_data_provider(
							&*client_clone,
							&parent,
						)?;

					Ok((slot, timestamp, storage_proof))
				}
			},
			force_authoring,
			backoff_authoring_blocks,
			babe_link,
			block_proposal_slot_portion: SlotProportion::new(0.5),
			max_block_proposal_slot_portion: None,
			telemetry: telemetry.as_ref().map(|x| x.handle()),
		};

		let babe = sc_consensus_babe::start_babe(babe_config)?;
		task_manager.spawn_essential_handle().spawn_blocking(
			"babe-proposer",
			Some("block-authoring"),
			babe,
		);
	}

	// Spawn authority discovery module.
	if role.is_authority() {
		let authority_discovery_role =
			sc_authority_discovery::Role::PublishAndDiscover(keystore_container.keystore());
		let dht_event_stream =
			network.event_stream("authority-discovery").filter_map(|e| async move {
				match e {
					Event::Dht(e) => Some(e),
					_ => None,
				}
			});
		let (authority_discovery_worker, _service) =
			sc_authority_discovery::new_worker_and_service_with_config(
				sc_authority_discovery::WorkerConfig {
					publish_non_global_ips: auth_disc_publish_non_global_ips,
					..Default::default()
				},
				client.clone(),
				network.clone(),
				Box::pin(dht_event_stream),
				authority_discovery_role,
				prometheus_registry.clone(),
			);

		task_manager.spawn_handle().spawn(
			"authority-discovery-worker",
			Some("networking"),
			authority_discovery_worker.run(),
		);
	}

	// if the node isn't actively participating in consensus then it doesn't
	// need a keystore, regardless of which protocol we use below.
	let keystore = if role.is_authority() { Some(keystore_container.keystore()) } else { None };

	let config = sc_consensus_grandpa::Config {
		// FIXME #1578 make this available through chainspec
		gossip_duration: std::time::Duration::from_millis(333),
		justification_generation_period: GRANDPA_JUSTIFICATION_PERIOD,
		name: Some(name),
		observer_enabled: false,
		keystore,
		local_role: role,
		telemetry: telemetry.as_ref().map(|x| x.handle()),
		protocol_name: grandpa_protocol_name,
	};

	if enable_grandpa {
		// start the full GRANDPA voter
		// NOTE: non-authorities could run the GRANDPA observer protocol, but at
		// this point the full voter should provide better guarantees of block
		// and vote data availability than the observer. The observer has not
		// been tested extensively yet and having most nodes in a network run it
		// could lead to finality stalls.
		let grandpa_config = sc_consensus_grandpa::GrandpaParams {
			config,
			link: grandpa_link,
			sync: Arc::new(sync_service),
			network: network.clone(),
			telemetry: telemetry.as_ref().map(|x| x.handle()),
			voting_rule: sc_consensus_grandpa::VotingRulesBuilder::default().build(),
			prometheus_registry,
			shared_voter_state,
			offchain_tx_pool_factory: OffchainTransactionPoolFactory::new(transaction_pool),
		};

		// the GRANDPA voter task is considered infallible, i.e.
		// if it fails we take down the service with it.
		task_manager.spawn_essential_handle().spawn_blocking(
			"grandpa-voter",
			None,
			sc_consensus_grandpa::run_grandpa_voter(grandpa_config)?,
		);
	}

	network_starter.start_network();
	Ok(NewFull { task_manager, client, backend, network, rpc_handlers })
}

struct RevertConsensus {
	blocks: BlockNumber,
	backend: Arc<FullBackend>,
}

impl ExecuteWithClient for RevertConsensus {
	type Output = sp_blockchain::Result<()>;

	fn execute_with_client<Client, Api, Backend>(self, client: Arc<Client>) -> Self::Output
	where
		Backend: sc_client_api::Backend<Block>,
		Backend::State: sp_api::StateBackend<BlakeTwo256>,
		Api: RuntimeApiCollection,
		Client: AbstractClient<Block, Backend, Api = Api> + 'static,
	{
		sc_consensus_babe::revert(client.clone(), self.backend, self.blocks)?;
		sc_consensus_grandpa::revert(client, self.blocks)?;
		Ok(())
	}
}

macro_rules! chain_ops {
	($config:expr; $scope:ident, $executor:ident, $variant:ident) => {{
		let config = $config;
		let basics = new_partial_basics::<$scope::RuntimeApi, $executor>(config)?;

		let sc_service::PartialComponents { client, backend, import_queue, task_manager, .. } =
			new_partial::<$scope::RuntimeApi, $executor>(&config, basics)?;
		Ok((Arc::new(Client::$variant(client)), backend, import_queue, task_manager))
	}};
}

#[allow(clippy::type_complexity)]
pub fn new_chain_ops(
	config: &Configuration,
) -> Result<
	(Arc<Client>, Arc<FullBackend>, sc_consensus::BasicQueue<Block>, TaskManager),
	ServiceError,
> {
	#[cfg(feature = "cere-dev-native")]
	if config.chain_spec.is_cere_dev() {
		return chain_ops!(config; cere_dev_runtime, CereDevExecutorDispatch, CereDev);
	}

	#[cfg(feature = "cere-native")]
	{
		chain_ops!(config; cere_runtime, CereExecutorDispatch, Cere)
	}

	#[cfg(not(feature = "cere-native"))]
	Err(Error::NoRuntime)
}

/// Can be called for a `Configuration` to identify which network the configuration targets.
pub trait IdentifyVariant {
	/// Returns if this is a configuration for the `Cere` network.
	fn is_cere(&self) -> bool;

	/// Returns if this is a configuration for the `Cere Dev` network.
	fn is_cere_dev(&self) -> bool;
}

impl IdentifyVariant for Box<dyn sc_service::ChainSpec> {
	fn is_cere(&self) -> bool {
		self.id().starts_with("cere_mainnet") ||
			self.id().starts_with("cere_qanet") ||
			self.id().starts_with("cere_testnet")
	}
	fn is_cere_dev(&self) -> bool {
		// Works for "cere-devnet" and "dev" arms in the load_spec(...) call.
		// If you are specifying a customSpecRaw.json for "path" arm along with the
		// "--force-cere-dev" flag, make sure your spec has a compatible "id" field to satisfy this
		// condition
		self.id().starts_with("cere_dev")
	}
}

pub fn revert_backend(
	client: Arc<Client>,
	backend: Arc<FullBackend>,
	blocks: BlockNumber,
) -> Result<(), ServiceError> {
	client.execute_with(RevertConsensus { blocks, backend })?;
	Ok(())
}
