// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use libra_config::config::NodeConfig;
//use network::validator_network::{ConsensusNetworkEvents,
// ConsensusNetworkSender};

use super::chained_bft::chained_bft_consensus_provider::ChainedBftProvider;
//use executor::Executor;
use grpcio::EnvBuilder;
//use state_synchronizer::StateSyncClient;
use network::NetworkService;
use std::sync::Arc;
//use storage_client::{StorageRead, StorageReadServiceClient};
//use vm_runtime::LibraVM;
use super::super::executor::Executor;
use crate::{
    alliance_tree_graph::{
        consensus::TreeGraphConsensus,
        hsb_sync_protocol::sync_protocol::HotStuffSynchronizationProtocol,
    },
    sync::request_manager::RequestManager,
};
use cfx_types::H256;
use libra_types::transaction::SignedTransaction;
use primitives::TransactionWithSignature;

/// Public interface to a consensus protocol.
pub trait ConsensusProvider {
    /// Spawns new threads, starts the consensus operations (retrieve txns,
    /// consensus protocol, execute txns, commit txns, update txn status in
    /// the mempool, etc). The function returns after consensus has
    /// recovered its initial state, and has established the required
    /// connections (e.g., to mempool and executor).
    fn start(
        &mut self, network: Arc<NetworkService>, own_node_hash: H256,
        request_manager: Arc<RequestManager>,
    ) -> Result<()>;

    /// Stop the consensus operations. The function returns after graceful
    /// shutdown.
    fn stop(&mut self);
}

/// Helper function to create a ConsensusProvider based on configuration
pub fn make_consensus_provider(
    node_config: &mut NodeConfig,
    /*network_sender: ConsensusNetworkSender,
     *network_receiver: ConsensusNetworkEvents, */
    executor: Arc<Executor>,
    /* state_sync_client: Arc<StateSyncClient>, */
    tg_consensus: Arc<TreeGraphConsensus>,
) -> Box<dyn ConsensusProvider>
{
    Box::new(ChainedBftProvider::new(
        node_config,
        /*network_sender,
         *network_receiver, */
        executor,
        /* state_sync_client, */
        tg_consensus,
    ))
}

/*
/// Create a storage read client based on the config
pub fn create_storage_read_client(config: &NodeConfig) -> Arc<dyn StorageRead> {
    let env = Arc::new(EnvBuilder::new().name_prefix("grpc-con-sto-").build());
    Arc::new(StorageReadServiceClient::new(
        env,
        &config.storage.address,
        config.storage.port,
    ))
}
*/
