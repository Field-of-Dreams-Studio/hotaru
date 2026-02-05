//! SQL message and result types

use std::collections::HashMap;
use std::error::Error;
use akari::Value;
use bytes::BytesMut;

/// SQL message types for communication
#[derive(Debug, Clone)]
pub enum Message {
    /// Execute a query and return results
    Query(String),
    /// Execute a statement (INSERT, UPDATE, DELETE)
    Execute(String),
    /// Result of a query/execute
    Result(QueryResult),
}

/// SQL query result
#[derive(Debug, Clone, Default)]
pub struct QueryResult {
    /// Number of rows affected (for INSERT, UPDATE, DELETE)
    pub rows_affected: Option<u64>,
    /// Query result data as JSON-like values
    pub data: Vec<HashMap<String, Value>>,
}

impl QueryResult {
    /// Create a new empty result
    pub fn new() -> Self {
        Self::default()
    }

    /// Create a result with a single row
    pub fn single<T: serde::Serialize>(item: T) -> Self {
        match serde_json::to_value(item) {
            Ok(serde_json::Value::Object(map)) => {
                let mut row = HashMap::new();
                for (k, v) in map {
                    row.insert(k, json_to_akari_value(v));
                }
                Self {
                    rows_affected: Some(1),
                    data: vec![row],
                }
            }
            _ => Self::default(),
        }
    }

    /// Create a result with multiple rows
    pub fn rows<T: serde::Serialize>(items: Vec<T>) -> Self {
        let mut data = Vec::new();
        for item in items {
            if let Ok(serde_json::Value::Object(map)) = serde_json::to_value(item) {
                let mut row = HashMap::new();
                for (k, v) in map {
                    row.insert(k, json_to_akari_value(v));
                }
                data.push(row);
            }
        }
        let count = data.len() as u64;
        Self {
            rows_affected: Some(count),
            data,
        }
    }

    /// Create a result for execute operations (no data, just affected rows)
    pub fn affected(rows: u64) -> Self {
        Self {
            rows_affected: Some(rows),
            data: Vec::new(),
        }
    }

    /// Check if result is empty
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }

    /// Get number of rows
    pub fn len(&self) -> usize {
        self.data.len()
    }

    /// Get first row if it exists
    pub fn first(&self) -> Option<&HashMap<String, Value>> {
        self.data.first()
    }
}

/// Convert serde_json::Value to akari::Value
fn json_to_akari_value(v: serde_json::Value) -> Value {
    match v {
        serde_json::Value::Null => Value::new(""),
        serde_json::Value::Bool(b) => Value::new(b),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Value::new(i)
            } else if let Some(f) = n.as_f64() {
                Value::new(f)
            } else {
                Value::new(n.to_string())
            }
        }
        serde_json::Value::String(s) => Value::new(s),
        serde_json::Value::Array(arr) => {
            let v: Vec<Value> = arr.into_iter().map(json_to_akari_value).collect();
            Value::new(v)
        }
        serde_json::Value::Object(map) => {
            let mut obj = HashMap::new();
            for (k, v) in map {
                obj.insert(k, json_to_akari_value(v));
            }
            Value::new(obj)
        }
    }
}

impl crate::connection::protocol::Message for Message {
    fn encode(&self, _buf: &mut BytesMut) -> Result<(), Box<dyn Error + Send + Sync>> {
        // SQL messages don't need wire encoding
        Ok(())
    }

    fn decode(_buf: &mut BytesMut) -> Result<Option<Self>, Box<dyn Error + Send + Sync>>
    where
        Self: Sized,
    {
        // SQL messages don't need wire decoding
        Ok(None)
    }
}
