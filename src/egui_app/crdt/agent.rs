//! # CRDT Agent Management
//!
//! Manages local agents and unique ID generation for CRDT operations.
//! Each device has a unique agent ID that identifies operations originating from it.
//!
//! ## Features
//!
//! - **Unique Agent IDs**: Generate and manage unique agent identifiers
//! - **Lamport Timestamps**: Maintain causal ordering of operations
//! - **Operation Sequencing**: Ensure operations are properly ordered
//! - **Persistence**: Store agent state across application restarts
//!
//! ## Usage
//!
//! ```rust,no_run
//! use xfmail::egui_app::crdt::Agent;
//!
//! // Create or load agent
//! let agent = Agent::new();
//!
//! // Get next operation ID
//! let op_id = agent.next_operation_id();
//!
//! // Get current timestamp
//! let timestamp = agent.current_timestamp();
//!
//! // Advance timestamp after operation
//! agent.advance_timestamp();
//! ```

use std::sync::atomic::{AtomicU64, Ordering};
use uuid::Uuid;

/// CRDT Agent representing a device/user in the distributed system
#[derive(Debug)]
pub struct Agent {
    /// Unique agent identifier
    id: u64,
    /// Lamport timestamp for causal ordering
    timestamp: AtomicU64,
    /// Operation sequence number
    operation_counter: AtomicU64,
    /// Device UUID for additional uniqueness
    device_id: Uuid,
}

impl Agent {
    /// Create a new agent with a unique ID
    ///
    /// Generates a new agent ID and initializes timestamps.
    /// In a real implementation, this would load from persistent storage.
    pub fn new() -> Self {
        Self {
            id: Self::generate_agent_id(),
            timestamp: AtomicU64::new(0),
            operation_counter: AtomicU64::new(0),
            device_id: Uuid::new_v4(),
        }
    }

    /// Create agent with specific ID (for testing or restoration)
    pub fn with_id(id: u64) -> Self {
        Self {
            id,
            timestamp: AtomicU64::new(0),
            operation_counter: AtomicU64::new(0),
            device_id: Uuid::new_v4(),
        }
    }

    /// Get the agent's unique ID
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Get the device UUID
    pub fn device_id(&self) -> &Uuid {
        &self.device_id
    }

    /// Get the current Lamport timestamp
    pub fn current_timestamp(&self) -> u64 {
        self.timestamp.load(Ordering::SeqCst)
    }

    /// Advance the Lamport timestamp
    ///
    /// Should be called after each operation to maintain causal ordering.
    pub fn advance_timestamp(&self) {
        self.timestamp.fetch_add(1, Ordering::SeqCst);
    }

    /// Update timestamp to maximum of current and received timestamp
    ///
    /// Used when receiving operations from other agents.
    pub fn update_timestamp(&self, received_timestamp: u64) {
        let current = self.current_timestamp();
        if received_timestamp > current {
            self.timestamp.store(received_timestamp, Ordering::SeqCst);
        }
        // Always advance by 1 after receiving
        self.advance_timestamp();
    }

    /// Generate the next operation ID
    pub fn next_operation_id(&self) -> u64 {
        self.operation_counter.fetch_add(1, Ordering::SeqCst)
    }

    /// Generate a unique agent ID
    ///
    /// Uses a combination of timestamp and random data to ensure uniqueness.
    fn generate_agent_id() -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .hash(&mut hasher);
        Uuid::new_v4().hash(&mut hasher);
        hasher.finish()
    }

    /// Create an operation metadata for this agent
    pub fn create_operation_meta(
        &self,
        op_type: crate::egui_app::crdt::OperationType,
        data: Vec<u8>,
    ) -> crate::egui_app::crdt::OperationMeta {
        let op_id = self.next_operation_id();
        let timestamp = self.current_timestamp();

        crate::egui_app::crdt::OperationMeta {
            id: op_id,
            agent_id: self.id,
            timestamp,
            op_type,
            data,
        }
    }
}

impl Clone for Agent {
    fn clone(&self) -> Self {
        Self {
            id: self.id,
            timestamp: AtomicU64::new(self.timestamp.load(Ordering::SeqCst)),
            operation_counter: AtomicU64::new(self.operation_counter.load(Ordering::SeqCst)),
            device_id: self.device_id,
        }
    }
}

impl Default for Agent {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() {
        let agent1 = Agent::new();
        let agent2 = Agent::new();

        // Agents should have different IDs
        assert_ne!(agent1.id(), agent2.id());
        assert_ne!(agent1.device_id(), agent2.device_id());
    }

    #[test]
    fn test_timestamp_advancement() {
        let agent = Agent::new();

        let initial = agent.current_timestamp();
        agent.advance_timestamp();
        let after = agent.current_timestamp();

        assert_eq!(after, initial + 1);
    }

    #[test]
    fn test_timestamp_update() {
        let agent = Agent::new();

        agent.update_timestamp(100);
        assert_eq!(agent.current_timestamp(), 101); // 100 + 1

        agent.update_timestamp(50); // Should not go backwards
        assert_eq!(agent.current_timestamp(), 102); // Still advances
    }

    #[test]
    fn test_operation_id_generation() {
        let agent = Agent::new();

        let id1 = agent.next_operation_id();
        let id2 = agent.next_operation_id();

        assert_eq!(id2, id1 + 1);
    }

    #[test]
    fn test_operation_meta_creation() {
        let agent = Agent::new();
        let data = vec![1, 2, 3];

        let meta = agent.create_operation_meta(
            crate::egui_app::crdt::OperationType::Add,
            data.clone(),
        );

        assert_eq!(meta.agent_id, agent.id());
        assert_eq!(meta.op_type, crate::egui_app::crdt::OperationType::Add);
        assert_eq!(meta.data, data);
    }
}