use frame_support::dispatch::DispatchResult;
use frame_system::Config;
use sp_runtime::DispatchError;

use crate::{ClusterId, NodeParams, NodePubKey};

pub trait NodeVisitor<T: Config> {
	fn get_cluster_id(node_pub_key: &NodePubKey) -> Result<Option<ClusterId>, DispatchError>;
	fn exists(node_pub_key: &NodePubKey) -> bool;
	fn get_node_provider_id(node_pub_key: &NodePubKey) -> Result<T::AccountId, DispatchError>;
	fn get_node_params(node_pub_key: &NodePubKey) -> Result<NodeParams, DispatchError>;
	fn get_current_validator() -> T::AccountId; // todo! this needs to be moved to validator, as NodeVisitor is ddc node visitor
}

pub trait NodeCreator<T: Config> {
	fn create_node(
		node_pub_key: NodePubKey,
		provider_id: T::AccountId,
		node_params: NodeParams,
	) -> DispatchResult;
}
