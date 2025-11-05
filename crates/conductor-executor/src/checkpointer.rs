//! Checkpoint persistence for graph execution
//!
//! Provides SQLite-based persistence for workflow state, enabling:
//! - Multi-turn conversations
//! - Resume from interruption
//! - State history tracking
//!
//! Inspired by LangGraph's checkpoint system.

use anyhow::{Context, Result};
use conductor_primitives::State;
use rusqlite::{params, Connection};
use serde_json::Value;
use std::path::Path;

/// Checkpoint metadata
#[derive(Debug, Clone)]
pub struct Checkpoint {
    /// Workflow identifier
    pub workflow_id: String,

    /// Step number in the execution
    pub step: usize,

    /// Current node name
    pub node_name: String,

    /// State at this checkpoint
    pub state: State,

    /// Timestamp (Unix epoch)
    pub timestamp: i64,
}

/// Checkpointer manages workflow state persistence
///
/// Uses SQLite for reliable, embedded storage.
///
/// # Example
///
/// ```rust
/// use conductor_executor::Checkpointer;
/// use conductor_primitives::State;
///
/// # fn example() -> anyhow::Result<()> {
/// let checkpointer = Checkpointer::new(":memory:")?;
///
/// // Save checkpoint
/// let state = State::new();
/// checkpointer.save("workflow-123", 0, "start", &state)?;
///
/// // Load latest checkpoint
/// let (step, node, loaded_state) = checkpointer.load_latest("workflow-123")?;
/// # Ok(())
/// # }
/// ```
pub struct Checkpointer {
    conn: Connection,
}

impl Checkpointer {
    /// Create a new checkpointer with SQLite storage
    ///
    /// # Arguments
    /// * `db_path` - Path to SQLite database file, or ":memory:" for in-memory
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let conn = Connection::open(db_path).context("Failed to open SQLite database")?;

        let checkpointer = Self { conn };
        checkpointer.init_schema()?;

        Ok(checkpointer)
    }

    /// Initialize database schema
    fn init_schema(&self) -> Result<()> {
        self.conn
            .execute(
                "CREATE TABLE IF NOT EXISTS checkpoints (
                    id INTEGER PRIMARY KEY AUTOINCREMENT,
                    workflow_id TEXT NOT NULL,
                    step INTEGER NOT NULL,
                    node_name TEXT NOT NULL,
                    state_json TEXT NOT NULL,
                    timestamp INTEGER NOT NULL,
                    UNIQUE(workflow_id, step)
                )",
                [],
            )
            .context("Failed to create checkpoints table")?;

        // Create index for faster lookups
        self.conn
            .execute(
                "CREATE INDEX IF NOT EXISTS idx_workflow_step
                 ON checkpoints(workflow_id, step DESC)",
                [],
            )
            .context("Failed to create index")?;

        Ok(())
    }

    /// Save a checkpoint
    ///
    /// # Arguments
    /// * `workflow_id` - Unique workflow identifier
    /// * `step` - Step number (0-indexed)
    /// * `node_name` - Current node name
    /// * `state` - State to persist
    pub fn save(
        &self,
        workflow_id: &str,
        step: usize,
        node_name: &str,
        state: &State,
    ) -> Result<()> {
        let state_json = serde_json::to_string(state).context("Failed to serialize state")?;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        self.conn
            .execute(
                "INSERT OR REPLACE INTO checkpoints
                 (workflow_id, step, node_name, state_json, timestamp)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                params![workflow_id, step as i64, node_name, state_json, timestamp],
            )
            .context("Failed to insert checkpoint")?;

        Ok(())
    }

    /// Load the latest checkpoint for a workflow
    ///
    /// Returns (step, node_name, state)
    pub fn load_latest(&self, workflow_id: &str) -> Result<(usize, String, State)> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT step, node_name, state_json
                 FROM checkpoints
                 WHERE workflow_id = ?1
                 ORDER BY step DESC
                 LIMIT 1",
            )
            .context("Failed to prepare query")?;

        let checkpoint = stmt
            .query_row(params![workflow_id], |row| {
                Ok((
                    row.get::<_, i64>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            })
            .context("No checkpoint found for workflow")?;

        let step = checkpoint.0 as usize;
        let node_name = checkpoint.1;
        let state: State =
            serde_json::from_str(&checkpoint.2).context("Failed to deserialize state")?;

        Ok((step, node_name, state))
    }

    /// Load a specific checkpoint by step number
    pub fn load_at_step(&self, workflow_id: &str, step: usize) -> Result<(String, State)> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT node_name, state_json
                 FROM checkpoints
                 WHERE workflow_id = ?1 AND step = ?2",
            )
            .context("Failed to prepare query")?;

        let checkpoint = stmt
            .query_row(params![workflow_id, step as i64], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .context("Checkpoint not found at step")?;

        let node_name = checkpoint.0;
        let state: State =
            serde_json::from_str(&checkpoint.1).context("Failed to deserialize state")?;

        Ok((node_name, state))
    }

    /// List all checkpoints for a workflow
    pub fn list(&self, workflow_id: &str) -> Result<Vec<Checkpoint>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT step, node_name, state_json, timestamp
                 FROM checkpoints
                 WHERE workflow_id = ?1
                 ORDER BY step ASC",
            )
            .context("Failed to prepare query")?;

        let checkpoints = stmt
            .query_map(params![workflow_id], |row| {
                let step = row.get::<_, i64>(0)? as usize;
                let node_name = row.get::<_, String>(1)?;
                let state_json = row.get::<_, String>(2)?;
                let timestamp = row.get::<_, i64>(3)?;

                let state: State = serde_json::from_str(&state_json)
                    .map_err(|e| rusqlite::Error::ToSqlConversionFailure(Box::new(e)))?;

                Ok(Checkpoint {
                    workflow_id: workflow_id.to_string(),
                    step,
                    node_name,
                    state,
                    timestamp,
                })
            })
            .context("Failed to query checkpoints")?
            .collect::<Result<Vec<_>, _>>()
            .context("Failed to collect checkpoints")?;

        Ok(checkpoints)
    }

    /// Delete all checkpoints for a workflow
    pub fn delete_workflow(&self, workflow_id: &str) -> Result<usize> {
        let deleted = self
            .conn
            .execute(
                "DELETE FROM checkpoints WHERE workflow_id = ?1",
                params![workflow_id],
            )
            .context("Failed to delete checkpoints")?;

        Ok(deleted)
    }

    /// Count checkpoints for a workflow
    pub fn count(&self, workflow_id: &str) -> Result<usize> {
        let count: i64 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM checkpoints WHERE workflow_id = ?1",
                params![workflow_id],
                |row| row.get(0),
            )
            .context("Failed to count checkpoints")?;

        Ok(count as usize)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_checkpointer_creation() {
        let checkpointer = Checkpointer::new(":memory:");
        assert!(checkpointer.is_ok());
    }

    #[test]
    fn test_save_and_load() {
        let checkpointer = Checkpointer::new(":memory:").unwrap();

        let state = State::new().set("count", json!(42));

        checkpointer
            .save("test-workflow", 0, "start", &state)
            .unwrap();

        let (step, node, loaded_state) = checkpointer.load_latest("test-workflow").unwrap();

        assert_eq!(step, 0);
        assert_eq!(node, "start");
        assert_eq!(loaded_state.get("count"), Some(&json!(42)));
    }

    #[test]
    fn test_multiple_checkpoints() {
        let checkpointer = Checkpointer::new(":memory:").unwrap();

        // Save 3 checkpoints
        let state1 = State::new().set("step", json!(1));
        let state2 = State::new().set("step", json!(2));
        let state3 = State::new().set("step", json!(3));

        checkpointer.save("wf1", 0, "node_a", &state1).unwrap();
        checkpointer.save("wf1", 1, "node_b", &state2).unwrap();
        checkpointer.save("wf1", 2, "node_c", &state3).unwrap();

        // Load latest
        let (step, node, state) = checkpointer.load_latest("wf1").unwrap();
        assert_eq!(step, 2);
        assert_eq!(node, "node_c");
        assert_eq!(state.get("step"), Some(&json!(3)));

        // Load at specific step
        let (node, state) = checkpointer.load_at_step("wf1", 1).unwrap();
        assert_eq!(node, "node_b");
        assert_eq!(state.get("step"), Some(&json!(2)));
    }

    #[test]
    fn test_list_checkpoints() {
        let checkpointer = Checkpointer::new(":memory:").unwrap();

        let state = State::new();
        checkpointer.save("wf1", 0, "a", &state).unwrap();
        checkpointer.save("wf1", 1, "b", &state).unwrap();
        checkpointer.save("wf1", 2, "c", &state).unwrap();

        let checkpoints = checkpointer.list("wf1").unwrap();
        assert_eq!(checkpoints.len(), 3);
        assert_eq!(checkpoints[0].step, 0);
        assert_eq!(checkpoints[1].step, 1);
        assert_eq!(checkpoints[2].step, 2);
    }

    #[test]
    fn test_count_checkpoints() {
        let checkpointer = Checkpointer::new(":memory:").unwrap();

        assert_eq!(checkpointer.count("wf1").unwrap(), 0);

        let state = State::new();
        checkpointer.save("wf1", 0, "a", &state).unwrap();
        checkpointer.save("wf1", 1, "b", &state).unwrap();

        assert_eq!(checkpointer.count("wf1").unwrap(), 2);
    }

    #[test]
    fn test_delete_workflow() {
        let checkpointer = Checkpointer::new(":memory:").unwrap();

        let state = State::new();
        checkpointer.save("wf1", 0, "a", &state).unwrap();
        checkpointer.save("wf1", 1, "b", &state).unwrap();
        checkpointer.save("wf2", 0, "a", &state).unwrap();

        let deleted = checkpointer.delete_workflow("wf1").unwrap();
        assert_eq!(deleted, 2);

        assert_eq!(checkpointer.count("wf1").unwrap(), 0);
        assert_eq!(checkpointer.count("wf2").unwrap(), 1);
    }

    #[test]
    fn test_replace_checkpoint() {
        let checkpointer = Checkpointer::new(":memory:").unwrap();

        let state1 = State::new().set("version", json!(1));
        let state2 = State::new().set("version", json!(2));

        checkpointer.save("wf1", 0, "node", &state1).unwrap();
        checkpointer.save("wf1", 0, "node", &state2).unwrap();

        let (_, _, state) = checkpointer.load_latest("wf1").unwrap();
        assert_eq!(state.get("version"), Some(&json!(2)));

        // Should have only 1 checkpoint (replaced)
        assert_eq!(checkpointer.count("wf1").unwrap(), 1);
    }
}
