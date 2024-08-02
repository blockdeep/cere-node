// This file is part of Substrate.

// Copyright (C) 2019-2022 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Types and traits for interfacing between the host and the wasm runtime.

#![cfg_attr(not(feature = "std"), no_std)]

use std::{cell::RefCell, str, sync::Arc};

use cere_dev_runtime as cere_dev;
use cere_runtime as cere;
use cere_runtime::wasm_binary_unwrap;
use lazy_static::lazy_static;
use node_primitives::Block;
use parking_lot::ReentrantMutex;
use sc_client_api::{backend::Backend, HeaderBackend};
use sc_executor_common::wasm_runtime::WasmModule;
use sc_service::{new_db_backend, BasePath, Configuration};
use sp_api::{ProofRecorder, StorageTransactionCache};
use sp_core::traits::{Externalities, FetchRuntimeCode, RuntimeCode};
use sp_runtime::generic::BlockId;
use sp_runtime_interface::runtime_interface;
use sp_state_machine::{Ext, OverlayedChanges, StateMachine, StorageProof};
use sp_std::borrow::Cow;
use sp_wasm_interface::{FunctionContext, HostFunctions, Pointer};
use wasmi::{memory_units::Pages, MemoryInstance, TableInstance};

mod freeing_bump;

mod util;

mod sandbox_instance;
mod sandbox_wasmi_backend;

mod wasmi_executor;
use wasmi_executor::{create_runtime, FunctionExecutor, WasmiInstance};

pub fn get_runtime_code() -> Cow<'static, [u8]> {
	let db_config = sc_client_db::DatabaseSettings {
		trie_cache_maximum_size: Some(67108864),
		state_pruning: None,
		source: sc_client_db::DatabaseSource::RocksDb { path: std::path::PathBuf::from("/Users/yahortsaryk/work/blockchain-host-functions/data9-125421-run-1/chains/cere_mainnet/db/full"), cache_size: 1024 },
		blocks_pruning: sc_client_db::BlocksPruning::KeepFinalized,
	};

	let backend = new_db_backend::<Block>(db_config).expect("backend to be created");

	let mut overlay = OverlayedChanges::default();
	let at_hash = backend
		.blockchain()
		.expect_block_hash_from_id(&BlockId::number(125421))
		.expect("at_hash exists");

	let state = backend.state_at(at_hash).expect("state exists");
	let mut cache =
		StorageTransactionCache::<Block, sc_client_db::RefTrackingState<Block>>::default();

	let mut _ext = Ext::new(&mut overlay, &mut cache, &state, None);
	let state_runtime_code = sp_state_machine::backend::BackendRuntimeCode::new(&state);
	let runtime_code = state_runtime_code
		.runtime_code()
		.map_err(sp_blockchain::Error::RuntimeCode)
		.expect("runtime_code exists");

	let code = runtime_code.fetch_runtime_code().expect("Code to be fetched");
	Cow::Owned(code.to_vec())
}

pub fn create_wasmi_instance() -> WasmiInstance {
	// The runtime was at 266 version at block 125423 where the missing sandbox host functions were
	// applied.
	let runtime = &include_bytes!("./node_runtime_266.wasm")[..];

	log::info!(target: "wasm_binary_unwrap", "LENGHT OF WASM BINARY {} ", runtime.len());
	let blob = sc_executor_common::runtime_blob::RuntimeBlob::uncompress_if_needed(runtime.clone())
		.expect("Runtime Blob to be ok");
	let heap_pages = 2048;
	let allow_missing_func_imports = true;

	let mut host_functions = sp_io::SubstrateHostFunctions::host_functions();
	let sandbox_host_functions = crate::sandbox::HostFunctions::host_functions();
	let benchmarking_host_functions =
		frame_benchmarking::benchmarking::HostFunctions::host_functions();

	host_functions.extend(sandbox_host_functions);
	host_functions.extend(benchmarking_host_functions);

	let custom_host_fn = custom_host_functions::HostFunctions::host_functions();
	host_functions.extend(custom_host_fn);

	let runtime = create_runtime(blob, heap_pages, host_functions, allow_missing_func_imports)
		// .map(|runtime| -> Arc<dyn WasmModule> { Arc::new(runtime) })
		.expect("Runtime to be created");

	let runtime_wasmi_instance =
		runtime.new_wasmi_instance().expect("Runtime instance to be created");
	*runtime_wasmi_instance
}

lazy_static! {
	static ref WASMI_INSTANCE: ReentrantMutex<RefCell<WasmiInstance>> =
		ReentrantMutex::new(RefCell::new(create_wasmi_instance()));
	// static ref SANDBOX: ReentrantMutex<RefCell<FunctionExecutor>> = ReentrantMutex::new(
	// 	RefCell::new(WASMI_INSTANCE.lock().borrow().create_function_executor())
	// );
}

const LOG_TARGET: &str = "yahor-runtime-interface-sandbox";

/// Something that provides access to the sandbox.
#[runtime_interface(wasm_only)]
pub trait Sandbox {
	/// Instantiate a new sandbox instance with the given `wasm_code`.
	fn instantiate(
		&mut self,
		dispatch_thunk: u32,
		wasm_code: &[u8],
		env_def: &[u8],
		state_ptr: Pointer<u8>,
	) -> u32 {
		log::info!(target: LOG_TARGET, "instantiate START: dispatch_thunk={:?}, env_def={:?}, state_ptr={:?}", dispatch_thunk, env_def, state_ptr);
		// let lock = SANDBOX.lock();
		// let mut sandbox = lock.borrow_mut();
		let binding = WASMI_INSTANCE.lock();
		let mut binding = binding.borrow_mut();
		let mut sandbox = binding.create_function_executor_2();
		let ref_to_ctx: &mut dyn FunctionContext = *self;
		let arc_from_ref = Box::new(ref_to_ctx);
		sandbox.set_runtime_memory(arc_from_ref);

		let res = sandbox
			.instance_new(dispatch_thunk, wasm_code, env_def, state_ptr.into())
			.expect("Failed to instantiate a new sandbox");

		log::info!(target: LOG_TARGET, "instantiate END: dispatch_thunk={:?}, env_def={:?}, state_ptr={:?}", dispatch_thunk, env_def, state_ptr);

		res
	}

	/// Invoke an exported function by a name.
	fn invoke(
		&mut self,
		instance_idx: u32,
		function: &str,
		args: &[u8],
		return_val_ptr: Pointer<u8>,
		return_val_len: u32,
		state_ptr: Pointer<u8>,
	) -> u32 {
		log::info!(target: LOG_TARGET, "invoke START: instance_idx={:?}, function={:?}", instance_idx, function);
		// let lock = SANDBOX.lock();
		// let mut sandbox = lock.borrow_mut();
		let binding = WASMI_INSTANCE.lock();
		let mut binding = binding.borrow_mut();
		let mut sandbox = binding.create_function_executor_2();
		let ref_to_ctx: &mut dyn FunctionContext = *self;
		let arc_from_ref = Box::new(ref_to_ctx);
		sandbox.set_runtime_memory(arc_from_ref);

		let res = sandbox
			.invoke(instance_idx, function, args, return_val_ptr, return_val_len, state_ptr.into())
			.expect("Failed to invoke function with sandbox");

		log::info!(target: LOG_TARGET, "invoke END: instance_idx={:?}, function={:?}", instance_idx, function);

		res
	}

	/// Create a new memory instance with the given `initial` size and the `maximum` size.
	/// The size is given in wasm pages.
	fn memory_new(&mut self, initial: u32, maximum: u32) -> u32 {
		log::info!(target: LOG_TARGET, "memory_new START: initial={:?}, maximum={:?}", initial, maximum);
		// let lock = SANDBOX.lock();
		// let mut sandbox = lock.borrow_mut();
		let binding = WASMI_INSTANCE.lock();
		let mut binding = binding.borrow_mut();
		let mut sandbox = binding.create_function_executor_2();
		let ref_to_ctx: &mut dyn FunctionContext = *self;
		let arc_from_ref = Box::new(ref_to_ctx);
		sandbox.set_runtime_memory(arc_from_ref);

		let res = sandbox
			.memory_new(initial, maximum)
			.expect("Failed to create new memory with sandbox");

		log::info!(target: LOG_TARGET, "memory_new END: initial={:?}, maximum={:?}", initial, maximum);

		res
	}

	// /// Get sandbox memory from the `memory_id` instance at `offset` into the given buffer.
	// fn memory_get(
	// 	&mut self,
	// 	memory_idx: u32,
	// 	offset: u32,
	// 	buf_ptr: Pointer<u8>,
	// 	buf_len: u32,
	// ) -> u32 {
	// 	// let lock = SANDBOX.lock();
	// 	// let mut sandbox = lock.borrow_mut();
	// 	let binding = WASMI_INSTANCE.lock();
	// 	let mut binding = binding.borrow_mut();
	// 	let mut sandbox = binding.create_function_executor_2();
	// 	let ref_to_ctx: &mut dyn FunctionContext = *self;
	// 	let arc_from_ref = Box::new(ref_to_ctx);
	// 	sandbox.set_runtime_memory(arc_from_ref);

	// 	let mut vec = Vec::new();
	// 	let data = vec.as_mut_slice();

	// 	let res = sandbox
	// 		.memory_get(memory_idx, offset, data)
	// 		.expect("Failed to get memory with sandbox");

	// 	let _ = self.write_memory(buf_ptr, data).unwrap();

	// 	res
	// }

	// /// Set sandbox memory from the given value.
	// fn memory_set(
	// 	&mut self,
	// 	memory_idx: u32,
	// 	offset: u32,
	// 	val_ptr: Pointer<u8>,
	// 	val_len: u32,
	// ) -> u32 {
	// 	// let lock = SANDBOX.lock();
	// 	// let mut sandbox = lock.borrow_mut();
	// 	let binding = WASMI_INSTANCE.lock();
	// 	let mut binding = binding.borrow_mut();
	// 	let mut sandbox = binding.create_function_executor_2();
	// 	let data = self.read_memory(val_ptr, val_len).unwrap().clone();

	// 	let ref_to_ctx: &mut dyn FunctionContext = *self;
	// 	let arc_from_ref = Box::new(ref_to_ctx);
	// 	sandbox.set_runtime_memory(arc_from_ref);

	// 	sandbox
	// 		.memory_set(memory_idx, offset, &data)
	// 		.expect("Failed to set memory with sandbox")
	// }

	/// Get sandbox memory from the `memory_id` instance at `offset` into the given buffer.
	fn memory_get(
		&mut self,
		memory_idx: u32,
		offset: u32,
		buf_ptr: Pointer<u8>,
		buf_len: u32,
	) -> u32 {
		let binding = WASMI_INSTANCE.lock();
		let mut binding = binding.borrow_mut();
		let mut sandbox = binding.create_function_executor_2();
		let ref_to_ctx: &mut dyn FunctionContext = *self;
		let arc_from_ref = Box::new(ref_to_ctx);
		sandbox.set_runtime_memory(arc_from_ref);

		sandbox
			.memory_get(memory_idx, offset, buf_ptr, buf_len)
			.expect("Failed to get memory with sandbox")
	}

	/// Set sandbox memory from the given value.
	fn memory_set(
		&mut self,
		memory_idx: u32,
		offset: u32,
		val_ptr: Pointer<u8>,
		val_len: u32,
	) -> u32 {
		let binding = WASMI_INSTANCE.lock();
		let mut binding = binding.borrow_mut();
		let mut sandbox = binding.create_function_executor_2();

		let ref_to_ctx: &mut dyn FunctionContext = *self;
		let arc_from_ref = Box::new(ref_to_ctx);
		sandbox.set_runtime_memory(arc_from_ref);

		sandbox
			.memory_set(memory_idx, offset, val_ptr, val_len)
			.expect("Failed to set memory with sandbox")
	}

	/// Delete a memory instance.
	fn memory_teardown(&mut self, memory_idx: u32) {
		// let lock = SANDBOX.lock();
		// let mut sandbox = lock.borrow_mut();
		let binding = WASMI_INSTANCE.lock();
		let mut binding = binding.borrow_mut();
		let mut sandbox = binding.create_function_executor_2();
		let ref_to_ctx: &mut dyn FunctionContext = *self;
		let arc_from_ref = Box::new(ref_to_ctx);
		sandbox.set_runtime_memory(arc_from_ref);

		sandbox
			.memory_teardown(memory_idx)
			.expect("Failed to teardown memory with sandbox")
	}

	/// Delete a sandbox instance.
	fn instance_teardown(&mut self, instance_idx: u32) {
		// let lock = SANDBOX.lock();
		// let mut sandbox = lock.borrow_mut();
		let binding = WASMI_INSTANCE.lock();
		let mut binding = binding.borrow_mut();
		let mut sandbox = binding.create_function_executor_2();
		let ref_to_ctx: &mut dyn FunctionContext = *self;
		let arc_from_ref = Box::new(ref_to_ctx);
		sandbox.set_runtime_memory(arc_from_ref);

		sandbox
			.instance_teardown(instance_idx)
			.expect("Failed to teardown sandbox instance")
	}

	/// Get the value from a global with the given `name`. The sandbox is determined by the
	/// given `instance_idx` instance.
	fn get_global_val(
		&mut self,
		instance_idx: u32,
		name: &str,
	) -> Option<sp_wasm_interface::Value> {
		// let lock = SANDBOX.lock();
		// let sandbox = lock.borrow();
		let binding = WASMI_INSTANCE.lock();
		let mut binding = binding.borrow_mut();
		let mut sandbox = binding.create_function_executor_2();

		let ref_to_ctx: &mut dyn FunctionContext = *self;
		let arc_from_ref = Box::new(ref_to_ctx);
		sandbox.set_runtime_memory(arc_from_ref);

		sandbox
			.get_global_val(instance_idx, name)
			.expect("Failed to get global from sandbox")
	}
}

#[runtime_interface]
pub trait CustomHostFunctions {
	fn gas(amount: u64) {}
}
