// Copyright (c) The Libra Core Contributors
// SPDX-License-Identifier: Apache-2.0

#![forbid(unsafe_code)]

mod consensus_state;
mod error;
mod local_client;
mod persistent_storage;
mod safety_rules;
mod safety_rules_manager;
mod t_safety_rules;

pub use consensus_state::ConsensusState;
pub use error::Error;
pub use persistent_storage::{InMemoryStorage, OnDiskStorage};
pub use safety_rules::SafetyRules;
pub use safety_rules_manager::{SafetyRulesManager, SafetyRulesManagerConfig};
pub use t_safety_rules::TSafetyRules;

#[cfg(any(test, feature = "testing"))]
#[path = "test_utils.rs"]
pub mod test_utils;

#[cfg(test)]
#[path = "safety_rules_test.rs"]
mod safety_rules_test;