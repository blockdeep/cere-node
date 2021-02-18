### Cere Network DDC Pallet

### Dispatchable Functions
* `send_data` - Send string data to another account.

### Command to test
# Clone from git to /frame folder
cd ./frame
git clone https://github.com/Cerebellum-Network/ddc-pallet
# Check by command
cd ./ddc-pallet
SKIP_WASM_BUILD=1 cargo check

# Test by command
SKIP_WASM_BUILD=1 cargo test

### Import to node instruction
## Frame structure node:
In ./Cargo.toml add
[workspace]
members = [
	"bin/node-template/node",
    ...
    "frame/vesting",
	"frame/ddc-pallet",
	"primitives/allocator",
    ...
    "utils/wasm-builder",
]

In ./bin/node/runtime/Cargo.toml add
# frame dependencies
frame-executive = { version = "2.0.0", default-features = false, path = "../../../frame/executive" }
...
pallet-cere-ddc = { version = "2.0.0", default-features = false, path = "../../../frame/ddc-pallet" }

[features]
default = ["std"]
with-tracing = [ "frame-executive/with-tracing" ]
std = [
    ...
    "pallet-cere-ddc/std",
]

In .bin/node/runtime/src/lib.rs add
parameter_types! {
	// Minimum bounds on storage are important to secure your chain.
	pub const MinDataLength: usize = 1;
	// Maximum bounds on storage are important to secure your chain.
	pub const MaxDataLength: usize = usize::MAX;
}

/// Configure the send data pallet
impl pallet_cere_ddc::Trait for Runtime {
	type MinLength = MinDataLength;
	type MaxLength = MaxDataLength;
	// The ubiquitous event type.
	type Event = Event;
}
  
construct_runtime!(
	pub enum Runtime where
		Block = Block,
		NodeBlock = node_primitives::Block,
		UncheckedExtrinsic = UncheckedExtrinsic
	{
		System: frame_system::{Module, Call, Config, Storage, Event<T>},
        ...
        Multisig: pallet_multisig::{Module, Call, Storage, Event<T>},
        CereDDCModule: pallet_cere_ddc::{Module, Call, Storage, Event<T>},
	}
);

## Github:
In ./bin/node/runtime/Cargo.toml add
# frame dependencies
frame-executive = { version = "2.0.0", default-features = false, path = "../../../frame/executive" }
...
pallet-cere-ddc = { version = "2.0.0", default-features = false, path = "../../../frame/ddc-pallet" }

[features]
default = ["std"]
with-tracing = [ "frame-executive/with-tracing" ]
std = [
    ...
    "pallet-cere-ddc/std",
]