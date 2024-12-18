use sp_runtime_interface::runtime_interface;
use sp_wasm_interface::{Pointer, Result as SandboxResult, Value, WordSize};
pub type MemoryId = u32;
use sp_wasm_interface::Function;

#[cfg(feature = "std")]
use sp_externalities::ExternalitiesExt;

pub use sp_externalities::MultiRemovalResults;

/// Something that provides access to the sandbox.
#[runtime_interface(wasm_only)]
pub trait Sandbox {
	/// Get sandbox memory from the `memory_id` instance at `offset` into the given buffer.
	fn memory_get(
		&mut self,
		memory_idx: u32,
		offset: u32,
		buf_ptr: Pointer<u8>,
		buf_len: u32,
	) -> u32 {
		log::info!("Going through the memory_get function");
		return 0;
	}
	/// Set sandbox memory from the given value.
	fn memory_set(
		&mut self,
		memory_idx: u32,
		offset: u32,
		val_ptr: Pointer<u8>,
		val_len: u32,
	) -> u32 {
		log::info!("Going through the memory_set function");
		return 0;
	}
	/// Delete a memory instance.
	fn memory_teardown(&mut self, memory_idx: u32) {
		log::info!("Going through the memory_teardown function");
	}
	/// Create a new memory instance with the given `initial` size and the `maximum` size.
	/// The size is given in wasm pages.
	fn memory_new(&mut self, initial: u32, maximum: u32) -> u32 {
		log::info!("Going through the memory_new function");

		return 0;
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
		return 0;
	}
	/// Delete a sandbox instance.
	fn instance_teardown(&mut self, instance_idx: u32) {
		log::info!("Going through the instance_teardown function");
	}

	/// Get the value from a global with the given `name`. The sandbox is determined by the
	/// given `instance_idx` instance.
	///
	/// Returns `Some(_)` when the requested global variable could be found.
	fn get_global_val(
		&mut self,
		instance_idx: u32,
		name: &str,
	) -> Option<sp_wasm_interface::Value> {
		log::info!("Going through the get_global_val function");
		return Some(Value::I32(0));
	}

	/// Instantiate a new sandbox instance with the given `wasm_code`.
	fn instantiate(
		&mut self,
		dispatch_thunk: u32,
		wasm_code: &[u8],
		env_def: &[u8],
		state_ptr: Pointer<u8>,
	) -> u32 {
		log::info!("Going through the instantiate function");
		return 0;
	}
}
