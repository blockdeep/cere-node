pub mod v1 {
	use ddc_primitives::{
		ClusterId, ClusterNodeKind, ClusterNodeState, ClusterNodeStatus, ClusterNodesCount,
		ClusterNodesStats, ClusterStatus,
	};
	use frame_support::{log, pallet_prelude::*, traits::OnRuntimeUpgrade, weights::Weight};
	use sp_runtime::Saturating;
	use sp_std::collections::btree_map::BTreeMap;

	use crate::{
		cluster::{Cluster, ClusterProps},
		Clusters, ClustersNodes, ClustersNodesStats, Config, Pallet, LOG_TARGET,
	};

	#[derive(Decode)]
	pub struct OldCluster<AccountId> {
		pub cluster_id: ClusterId,
		pub manager_id: AccountId,
		pub reserve_id: AccountId,
		pub props: ClusterProps<AccountId>,
	}

	impl<AccountId> OldCluster<AccountId> {
		fn migrate_to_v1(self) -> Cluster<AccountId> {
			// all clusters are unbonded by default
			let status = ClusterStatus::Unbonded;
			Cluster {
				cluster_id: self.cluster_id,
				manager_id: self.manager_id,
				reserve_id: self.reserve_id,
				props: self.props,
				status,
			}
		}
	}

	pub type OldNodeStatus = bool;
	pub struct MigrateToV1<T>(sp_std::marker::PhantomData<T>);
	impl<T: Config> OnRuntimeUpgrade for MigrateToV1<T> {
		fn on_runtime_upgrade() -> Weight {
			let current_version = Pallet::<T>::current_storage_version();
			let onchain_version = Pallet::<T>::on_chain_storage_version();
			let mut weight = T::DbWeight::get().reads(1);

			if onchain_version == 0 && current_version == 1 {
				let mut translated_clusters = 0u64;
				Clusters::<T>::translate::<OldCluster<T::AccountId>, _>(
					|_cluster_id, old_cluster| {
						translated_clusters.saturating_inc();
						Some(old_cluster.migrate_to_v1())
					},
				);
				weight.saturating_accrue(
					T::DbWeight::get().reads_writes(translated_clusters, translated_clusters),
				);

				current_version.put::<Pallet<T>>();
				weight.saturating_accrue(T::DbWeight::get().writes(1));
				log::info!(
					target: LOG_TARGET,
					"Upgraded {} clusters, storage to version {:?}",
					translated_clusters,
					current_version
				);

				let mut translated_clusters_nodes = 0u64;
				let mut nodes_count_by_cluster: BTreeMap<ClusterId, ClusterNodesCount> =
					BTreeMap::new();
				ClustersNodes::<T>::translate::<OldNodeStatus, _>(
					|cluster_id, _node_pub_key, _old_value| {
						translated_clusters_nodes.saturating_inc();

						nodes_count_by_cluster
							.entry(cluster_id)
							.and_modify(|count| *count = count.saturating_add(1)) // If exists, update
							.or_insert(1); // If not, insert with a count of 1

						Some(ClusterNodeState {
							kind: ClusterNodeKind::External,
							status: ClusterNodeStatus::ValidationSucceeded,
							added_at: <frame_system::Pallet<T>>::block_number(),
						})
					},
				);
				weight.saturating_accrue(
					T::DbWeight::get()
						.reads_writes(translated_clusters_nodes, translated_clusters_nodes),
				);
				log::info!(
					target: LOG_TARGET,
					"Upgraded {} clusters nodes statuses, storage to version {:?}",
					translated_clusters_nodes,
					current_version
				);

				for (cluster_id, nodes_count) in nodes_count_by_cluster.iter() {
					ClustersNodesStats::<T>::insert(
						cluster_id,
						ClusterNodesStats {
							await_validation: 0,
							validation_succeeded: *nodes_count,
							validation_failed: 0,
						},
					);
					weight.saturating_accrue(T::DbWeight::get().writes(1));
				}
				log::info!(
					target: LOG_TARGET,
					"Upgraded {} clusters statistics, storage to version {:?}",
					nodes_count_by_cluster.len(),
					current_version
				);

				weight
			} else {
				log::info!(
					target: LOG_TARGET,
					"Migration did not execute. This probably should be removed"
				);

				weight
			}
		}

		#[cfg(feature = "try-runtime")]
		fn pre_upgrade() -> Result<Vec<u8>, &'static str> {
			frame_support::ensure!(
				Pallet::<T>::on_chain_storage_version() == 0,
				"must upgrade linearly"
			);
			let pre_clusters_count = Clusters::<T>::iter().count();
			let pre_clusters_nodes_count = ClustersNodes::<T>::iter().count();
			let pre_clusters_nodes_stats_count = ClustersNodesStats::<T>::iter().count();

			assert_eq!(
				pre_clusters_nodes_stats_count, 0,
				"clusters statistics should be empty before the migration"
			);

			Ok((pre_clusters_count as u32, pre_clusters_nodes_count as u32).encode())
		}

		#[cfg(feature = "try-runtime")]
		fn post_upgrade(state: Vec<u8>) -> Result<(), &'static str> {
			let (pre_clusters_count, pre_clusters_nodes_count): (u32, u32) = Decode::decode(
				&mut &state[..],
			)
			.expect("the state parameter should be something that was generated by pre_upgrade");

			let post_clusters_count = Clusters::<T>::iter().count() as u32;
			assert_eq!(
				pre_clusters_count, post_clusters_count,
				"the clusters count before and after the migration should be the same"
			);
			let post_clusters_nodes_count = ClustersNodes::<T>::iter().count() as u32;
			assert_eq!(
				pre_clusters_nodes_count, post_clusters_nodes_count,
				"the clusters nodes count before and after the migration should be the same"
			);

			let post_clusters_nodes_stats_count = ClustersNodesStats::<T>::iter().count() as u32;
			assert_eq!(
				post_clusters_count, post_clusters_nodes_stats_count,
				"the clusters statistics should be equal to clusters count after the migration"
			);

			let current_version = Pallet::<T>::current_storage_version();
			let onchain_version = Pallet::<T>::on_chain_storage_version();

			frame_support::ensure!(current_version == 1, "must_upgrade");
			assert_eq!(
				current_version, onchain_version,
				"after migration, the current_version and onchain_version should be the same"
			);

			Clusters::<T>::iter().for_each(|(_id, cluster)| {
				assert!(
					cluster.status == ClusterStatus::Unbonded,
					"cluster status should only be 'Unbonded'."
				)
			});
			Ok(())
		}
	}
}
