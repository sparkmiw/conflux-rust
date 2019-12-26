// Copyright 2019 Conflux Foundation. All rights reserved.
// Conflux is free software and distributed under GNU General Public License.
// See http://www.gnu.org/licenses/

use crate::{
    storage::{
        state_manager::StateManager,
        storage_db::{SnapshotDbManagerTrait, SnapshotInfo},
        FullSyncVerifier, SnapshotDbManagerSqlite,
    },
    sync::state::storage::{Chunk, ChunkKey},
};
use cfx_types::H256;
use primitives::{EpochId, MerkleHash};
use std::sync::{
    atomic::{AtomicUsize, Ordering::Relaxed},
    Arc,
};

pub struct Restorer {
    pub snapshot_epoch_id: EpochId,
    pub snapshot_merkle_root: MerkleHash,

    /// The verifier for chunks.
    /// Initialized after receiving a valid manifest.
    verifier: Option<FullSyncVerifier<SnapshotDbManagerSqlite>>,
}

impl Default for Restorer {
    fn default() -> Self { Self::new_with_default_root_dir(H256::zero()) }
}

impl Restorer {
    pub fn new_with_default_root_dir(checkpoint: H256) -> Self {
        Self::new(checkpoint)
    }

    pub fn new(checkpoint: H256) -> Self {
        Restorer {
            snapshot_epoch_id: checkpoint,
            snapshot_merkle_root: Default::default(),
            verifier: None,
        }
    }

    pub fn initialize_verifier(
        &mut self, verifier: FullSyncVerifier<SnapshotDbManagerSqlite>,
    ) {
        self.verifier = Some(verifier);
    }

    /// Append a chunk for restoration.
    pub fn append(&mut self, key: ChunkKey, chunk: Chunk) -> bool {
        match &mut self.verifier {
            // Not waiting for chunks
            None => false,
            Some(verifier) => {
                match verifier.restore_chunk(
                    &key.upper_bound_excl,
                    &chunk.keys,
                    chunk.values,
                ) {
                    Ok(true) => true,
                    _ => false,
                }
            }
        }
    }

    /// Start to restore chunks asynchronously.
    pub fn finalize_restoration(
        &self, state_manager: Arc<StateManager>, snapshot_info: SnapshotInfo,
    ) {
        state_manager
            .get_storage_manager()
            .get_snapshot_manager()
            .get_snapshot_db_manager()
            .finalize_full_sync_snapshot(
                &self.snapshot_epoch_id,
                &self.snapshot_merkle_root,
            )
            .expect("Fail to finalize full sync");
        state_manager
            .get_storage_manager()
            .register_new_snapshot(snapshot_info, true);

        debug!("complete to restore snapshot chunks");
    }

    pub fn restored_state_root(
        &self, _state_manager: Arc<StateManager>,
    ) -> MerkleHash {
        // TODO Double check the restored snapshot merkle root
        // But if all chunks pass the verification, it should be the same as
        // the this snapshot_merkle_root
        self.snapshot_merkle_root
    }
}

#[derive(Default, Debug)]
pub struct RestoreProgress {
    total: AtomicUsize,
    completed: AtomicUsize,
}

impl RestoreProgress {
    pub fn is_completed(&self) -> bool {
        let total = self.total.load(Relaxed);
        let completed = self.completed.load(Relaxed);
        completed >= total
    }
}
