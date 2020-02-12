// Copyright 2019 Conflux Foundation. All rights reserved.
// Conflux is free software and distributed under GNU General Public License.
// See http://www.gnu.org/licenses/

use delegate::delegate;

use crate::rpc::{
    impls::{cfx::RpcImplConfiguration, common::RpcImpl as CommonImpl},
    traits::{cfx::Cfx, debug::DebugRpc, test::TestRpc},
    types::{
        Account as RpcAccount, BFTStates, BlameInfo, Block as RpcBlock,
        BlockHashOrEpochNumber, Bytes, CallRequest, ConsensusGraphStates,
        EpochNumber, Filter as RpcFilter, Log as RpcLog, Receipt as RpcReceipt,
        SendTxRequest, Status as RpcStatus, SyncGraphStates,
        Transaction as RpcTransaction, H160 as RpcH160, H256 as RpcH256,
        H520 as RpcH520, U128 as RpcU128, U256 as RpcU256, U64 as RpcU64,
    },
};
use cfx_types::H256;
use cfxcore::{
    alliance_tree_graph::{
        blockgen::TGBlockGenerator, consensus::TreeGraphConsensus,
    },
    state_exposer::STATE_EXPOSER,
    PeerInfo, SharedConsensusGraph, SharedSynchronizationService,
    SharedTransactionPool,
};
use jsonrpc_core::{BoxFuture, Error as RpcError, Result as RpcResult};
use network::{
    node_table::{Node, NodeId},
    throttling, SessionDetails, UpdateNodeOperation,
};
use primitives::{SignedTransaction, TransactionWithSignature};
use std::{collections::BTreeMap, net::SocketAddr, sync::Arc};

pub struct RpcImpl {
    config: RpcImplConfiguration,
    pub consensus: SharedConsensusGraph,
    sync: SharedSynchronizationService,
    block_gen: Arc<TGBlockGenerator>,
    tx_pool: SharedTransactionPool,
    // tx_gen: Arc<TransactionGenerator>,
}

impl RpcImpl {
    pub fn new(
        consensus: SharedConsensusGraph, sync: SharedSynchronizationService,
        block_gen: Arc<TGBlockGenerator>, tx_pool: SharedTransactionPool,
        /* tx_gen: Arc<TransactionGenerator>, */
        config: RpcImplConfiguration,
    ) -> Self
    {
        RpcImpl {
            consensus,
            sync,
            block_gen,
            tx_pool,
            // tx_gen,
            config,
        }
    }

    fn consensus_graph_state(&self) -> RpcResult<ConsensusGraphStates> {
        let consensus_graph_states =
            STATE_EXPOSER.consensus_graph.lock().retrieve();
        Ok(ConsensusGraphStates::new(consensus_graph_states))
    }

    fn sync_graph_state(&self) -> RpcResult<SyncGraphStates> {
        let sync_graph_states = STATE_EXPOSER.sync_graph.lock().retrieve();
        Ok(SyncGraphStates::new(sync_graph_states))
    }

    fn bft_state(&self) -> RpcResult<BFTStates> {
        let bft_states = STATE_EXPOSER.bft.lock().retrieve();
        Ok(BFTStates::new(bft_states))
    }

    fn current_sync_phase(&self) -> RpcResult<String> {
        Ok(self.sync.current_sync_phase().name().into())
    }
}

pub struct CfxHandler {
    common: Arc<CommonImpl>,
    rpc_impl: Arc<RpcImpl>,
}

impl CfxHandler {
    pub fn new(common: Arc<CommonImpl>, rpc_impl: Arc<RpcImpl>) -> Self {
        CfxHandler { common, rpc_impl }
    }
}

impl Cfx for CfxHandler {
    delegate! {
        target self.common {
            fn blocks_by_epoch(&self, num: EpochNumber) -> RpcResult<Vec<RpcH256>>;
            fn best_block_hash(&self) -> RpcResult<RpcH256>;
        }

        /*target self.rpc_impl {
        }*/
    }

    not_supported! {
        fn block_by_epoch_number(&self, epoch_num: EpochNumber, include_txs: bool) -> RpcResult<RpcBlock>;
        fn block_by_hash_with_pivot_assumption(&self, block_hash: RpcH256, pivot_hash: RpcH256, epoch_number: RpcU64) -> RpcResult<RpcBlock>;
        fn block_by_hash(&self, hash: RpcH256, include_txs: bool) -> RpcResult<Option<RpcBlock>>;
        fn epoch_number(&self, epoch_num: Option<EpochNumber>) -> RpcResult<RpcU256>;
        fn gas_price(&self) -> RpcResult<RpcU256>;
        fn transaction_count(&self, address: RpcH160, num: Option<BlockHashOrEpochNumber>) -> RpcResult<RpcU256>;

        fn account(&self, address: RpcH160, num: Option<EpochNumber>) -> BoxFuture<RpcAccount>;
        fn balance(&self, address: RpcH160, num: Option<EpochNumber>) -> BoxFuture<RpcU256>;
        fn bank_balance(&self, address: RpcH160, num: Option<EpochNumber>) -> BoxFuture<RpcU256>;
        fn storage_balance(&self, address: RpcH160, num: Option<EpochNumber>) -> BoxFuture<RpcU256>;
        fn call(&self, request: CallRequest, epoch: Option<EpochNumber>) -> RpcResult<Bytes>;
        fn code(&self, address: RpcH160, epoch_num: Option<EpochNumber>) -> BoxFuture<Bytes>;
        fn estimate_gas(&self, request: CallRequest, epoch_num: Option<EpochNumber>) -> RpcResult<RpcU256>;
        fn get_logs(&self, filter: RpcFilter) -> BoxFuture<Vec<RpcLog>>;
        fn send_raw_transaction(&self, raw: Bytes) -> RpcResult<RpcH256>;
        fn transaction_by_hash(&self, hash: RpcH256) -> BoxFuture<Option<RpcTransaction>>;
        fn transaction_receipt(&self, tx_hash: RpcH256) -> BoxFuture<Option<RpcReceipt>>;

        fn interest_rate(&self, num: Option<EpochNumber>) -> RpcResult<RpcU256>;
        fn accumulate_interest_rate(&self, num: Option<EpochNumber>) -> RpcResult<RpcU256>;
    }
}

pub struct TestRpcImpl {
    common: Arc<CommonImpl>,
    rpc_impl: Arc<RpcImpl>,
}

impl TestRpcImpl {
    pub fn new(common: Arc<CommonImpl>, rpc_impl: Arc<RpcImpl>) -> Self {
        TestRpcImpl { common, rpc_impl }
    }
}

impl TestRpc for TestRpcImpl {
    delegate! {
        target self.common {
            fn add_latency(&self, id: NodeId, latency_ms: f64) -> RpcResult<()>;
            fn add_peer(&self, node_id: NodeId, address: SocketAddr) -> RpcResult<()>;
            fn drop_peer(&self, node_id: NodeId, address: SocketAddr) -> RpcResult<()>;
            fn get_block_count(&self) -> RpcResult<u64>;
            fn get_nodeid(&self, challenge: Vec<u8>) -> RpcResult<Vec<u8>>;
            fn get_peer_info(&self) -> RpcResult<Vec<PeerInfo>>;
            fn get_status(&self) -> RpcResult<RpcStatus>;
            fn say_hello(&self) -> RpcResult<String>;
            fn stop(&self) -> RpcResult<()>;
            fn save_node_db(&self) -> RpcResult<()>;
        }
    }

    not_supported! {
        fn chain(&self) -> RpcResult<Vec<RpcBlock>>;
        fn get_goodput(&self) -> RpcResult<String>;
        fn get_transaction_receipt(&self, tx_hash: H256) -> RpcResult<Option<RpcReceipt>>;

        fn expire_block_gc(&self, timeout: u64) -> RpcResult<()>;
        fn generate_block_with_blame_info(&self, num_txs: usize, block_size_limit: usize, blame_info: BlameInfo) -> RpcResult<H256>;
        fn generate_block_with_fake_txs(&self, raw_txs_without_data: Bytes, adaptive: Option<bool>, tx_data_len: Option<usize>) -> RpcResult<H256>;
        fn generate_custom_block(&self, parent_hash: H256, referee: Vec<H256>, raw_txs: Bytes, adaptive: Option<bool>) -> RpcResult<H256>;
        fn generate_fixed_block(&self, parent_hash: H256, referee: Vec<H256>, num_txs: usize, adaptive: bool, difficulty: Option<u64>) -> RpcResult<H256>;
        fn generate_one_block_special(&self, num_txs: usize, block_size_limit: usize, num_txs_simple: usize, num_txs_erc20: usize) -> RpcResult<()>;
        fn generate_block_with_nonce_and_timestamp(&self, parent: H256, referees: Vec<H256>, raw: Bytes, nonce: u64, timestamp: u64, adaptive: bool) -> RpcResult<H256>;
        fn generate_one_block(&self, num_txs: usize, block_size_limit: usize) -> RpcResult<H256>;
        fn generate(&self, num_blocks: usize, num_txs: usize) -> RpcResult<Vec<H256>>;
        fn send_usable_genesis_accounts(& self, account_start_index: usize) -> RpcResult<Bytes>;
        fn get_block_status(&self, block_hash: H256) -> RpcResult<(u8, bool)>;
        fn set_db_crash(&self, crash_probability: f64, crash_exit_code: i32) -> RpcResult<()>;
    }
}

pub struct DebugRpcImpl {
    common: Arc<CommonImpl>,
    rpc_impl: Arc<RpcImpl>,
}

impl DebugRpcImpl {
    pub fn new(common: Arc<CommonImpl>, rpc_impl: Arc<RpcImpl>) -> Self {
        DebugRpcImpl { common, rpc_impl }
    }
}

impl DebugRpc for DebugRpcImpl {
    delegate! {
        target self.common {
            fn clear_tx_pool(&self) -> RpcResult<()>;
            fn net_node(&self, id: NodeId) -> RpcResult<Option<(String, Node)>>;
            fn net_disconnect_node(&self, id: NodeId, op: Option<UpdateNodeOperation>) -> RpcResult<Option<usize>>;
            fn net_sessions(&self, node_id: Option<NodeId>) -> RpcResult<Vec<SessionDetails>>;
            fn net_throttling(&self) -> RpcResult<throttling::Service>;
            fn tx_inspect(&self, hash: RpcH256) -> RpcResult<BTreeMap<String, String>>;
            fn txpool_content(&self) -> RpcResult<BTreeMap<String, BTreeMap<String, BTreeMap<usize, Vec<RpcTransaction>>>>>;
            fn txpool_inspect(&self) -> RpcResult<BTreeMap<String, BTreeMap<String, BTreeMap<usize, Vec<String>>>>>;
            fn txpool_status(&self) -> RpcResult<BTreeMap<String, usize>>;
            fn accounts(&self) -> RpcResult<Vec<RpcH160>>;
            fn new_account(&self, password: String) -> RpcResult<RpcH160>;
            fn unlock_account(&self, address: RpcH160, password: String, duration: Option<RpcU128>) -> RpcResult<bool>;
            fn lock_account(&self, address: RpcH160) -> RpcResult<bool>;
            fn sign(&self, data: Bytes, address: RpcH160, password: Option<String>) -> RpcResult<RpcH520>;
        }

        target self.rpc_impl {
            fn current_sync_phase(&self) -> RpcResult<String>;
            fn consensus_graph_state(&self) -> RpcResult<ConsensusGraphStates>;
            fn sync_graph_state(&self) -> RpcResult<SyncGraphStates>;
            fn bft_state(&self) -> RpcResult<BFTStates>;
        }
    }

    not_supported! {
        fn send_transaction(&self, tx: SendTxRequest, password: Option<String>) -> BoxFuture<RpcH256>;
    }
}