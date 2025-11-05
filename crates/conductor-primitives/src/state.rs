//! Immutable state management with reducer pattern
//!
//! Inspired by LangGraph's state management, this module provides:
//! - Immutable state updates
//! - Reducer functions for customizable merge logic
//! - Type-safe state operations

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashMap;

use crate::message::Message;

/// Immutable state container
///
/// State is a key-value store where values can be any JSON-serializable type.
/// Updates create new state instances, preserving immutability.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct State {
    data: HashMap<String, Value>,
}

impl State {
    /// Create a new empty state
    pub fn new() -> Self {
        Self {
            data: HashMap::new(),
        }
    }

    /// Create state from a HashMap
    pub fn from_map(data: HashMap<String, Value>) -> Self {
        Self { data }
    }

    /// Get a value from the state
    pub fn get(&self, key: &str) -> Option<&Value> {
        self.data.get(key)
    }

    /// Get a typed value from the state
    pub fn get_typed<T: for<'de> Deserialize<'de>>(&self, key: &str) -> Option<T> {
        self.data.get(key).and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    /// Get messages from state (common operation)
    pub fn get_messages(&self) -> Vec<Message> {
        self.get_typed("messages").unwrap_or_default()
    }

    /// Set a value, returning a new state (immutable update)
    pub fn set(&self, key: impl Into<String>, value: Value) -> Self {
        let mut new_data = self.data.clone();
        new_data.insert(key.into(), value);
        Self { data: new_data }
    }

    /// Update state using a reducer
    pub fn update(&self, key: &str, new_value: Value, reducer: &dyn Reducer) -> Self {
        let current_value = self.get(key).cloned().unwrap_or(Value::Null);
        let merged_value = reducer.reduce(current_value, new_value);
        self.set(key, merged_value)
    }

    /// Merge another state into this one using reducers
    pub fn merge(&self, other: &State, reducers: &HashMap<String, Box<dyn Reducer>>) -> Self {
        let mut new_data = self.data.clone();

        for (key, new_value) in &other.data {
            let merged_value = if let Some(reducer) = reducers.get(key) {
                let current_value = self.get(key).cloned().unwrap_or(Value::Null);
                reducer.reduce(current_value, new_value.clone())
            } else {
                // Default: replace
                new_value.clone()
            };
            new_data.insert(key.clone(), merged_value);
        }

        Self { data: new_data }
    }

    /// Get all keys
    pub fn keys(&self) -> Vec<&String> {
        self.data.keys().collect()
    }

    /// Check if state is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get the number of keys
    pub fn len(&self) -> usize {
        self.data.len()
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new()
    }
}

impl From<HashMap<String, Value>> for State {
    fn from(data: HashMap<String, Value>) -> Self {
        Self::from_map(data)
    }
}

/// Reducer trait for customizing state merge logic
///
/// Reducers define how new values are combined with existing values.
/// This is crucial for operations like appending to lists or merging objects.
pub trait Reducer: Send + Sync {
    /// Reduce current and new values into a single value
    fn reduce(&self, current: Value, new: Value) -> Value;
}

/// Replace reducer: new value replaces current value
#[derive(Debug, Clone)]
pub struct ReplaceReducer;

impl Reducer for ReplaceReducer {
    fn reduce(&self, _current: Value, new: Value) -> Value {
        new
    }
}

/// Append reducer: append new value to current array
#[derive(Debug, Clone)]
pub struct AppendReducer;

impl Reducer for AppendReducer {
    fn reduce(&self, current: Value, new: Value) -> Value {
        match (current, new) {
            (Value::Array(mut arr), Value::Array(mut new_arr)) => {
                arr.append(&mut new_arr);
                Value::Array(arr)
            }
            (Value::Array(mut arr), new_val) => {
                arr.push(new_val);
                Value::Array(arr)
            }
            (Value::Null, Value::Array(arr)) => Value::Array(arr),
            (Value::Null, new_val) => Value::Array(vec![new_val]),
            (_, new_val) => new_val, // Fallback to replace
        }
    }
}

/// Merge reducer: deeply merge objects
#[derive(Debug, Clone)]
pub struct MergeReducer;

impl Reducer for MergeReducer {
    fn reduce(&self, current: Value, new: Value) -> Value {
        match (current, new) {
            (Value::Object(mut current_obj), Value::Object(new_obj)) => {
                for (key, value) in new_obj {
                    current_obj.insert(key, value);
                }
                Value::Object(current_obj)
            }
            (_, new_val) => new_val, // Fallback to replace
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_state_creation() {
        let state = State::new();
        assert!(state.is_empty());
        assert_eq!(state.len(), 0);
    }

    #[test]
    fn test_state_immutability() {
        let state1 = State::new();
        let state2 = state1.set("key", json!("value"));

        assert!(state1.is_empty());
        assert_eq!(state2.get("key"), Some(&json!("value")));
    }

    #[test]
    fn test_replace_reducer() {
        let state = State::new();
        let reducer = ReplaceReducer;
        let new_state = state.update("count", json!(42), &reducer);

        assert_eq!(new_state.get("count"), Some(&json!(42)));
    }

    #[test]
    fn test_append_reducer() {
        let state = State::new();
        let reducer = AppendReducer;

        let state = state.update("items", json!([1, 2]), &reducer);
        let state = state.update("items", json!(3), &reducer);

        assert_eq!(state.get("items"), Some(&json!([1, 2, 3])));
    }

    #[test]
    fn test_append_reducer_from_null() {
        let state = State::new();
        let reducer = AppendReducer;
        let state = state.update("items", json!(1), &reducer);

        assert_eq!(state.get("items"), Some(&json!([1])));
    }

    #[test]
    fn test_merge_reducer() {
        let state = State::new().set("config", json!({"a": 1}));
        let reducer = MergeReducer;
        let state = state.update("config", json!({"b": 2}), &reducer);

        assert_eq!(state.get("config"), Some(&json!({"a": 1, "b": 2})));
    }

    #[test]
    fn test_get_typed() {
        let state = State::new().set("count", json!(42));
        let count: i32 = state.get_typed("count").unwrap();
        assert_eq!(count, 42);
    }

    #[test]
    fn test_get_messages() {
        let messages = vec![
            Message::user("Hello"),
            Message::assistant("Hi there!"),
        ];
        let state = State::new().set("messages", json!(messages));

        let retrieved = state.get_messages();
        assert_eq!(retrieved.len(), 2);
        assert_eq!(retrieved[0].content, "Hello");
    }
}
