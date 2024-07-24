use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio_postgres::Row;

use crate::config::TaskTableLayout;

#[derive(Serialize, Deserialize)]
pub struct Task {
    pub id: i32,
    pub version: String,
    pub name: String,
    pub parameters: Value,
    pub metadata: Value,
}

impl Task {
    pub fn from_row_with_layout(row: Row, layout: &TaskTableLayout) -> Self {
        Self {
            id: row.get(layout.id.as_str()),
            version: row.get(layout.version.as_str()),
            name: row.get(layout.name.as_str()),
            parameters: row.get(layout.parameters.as_str()),
            metadata: row.get(layout.metadata.as_str()),
        }
    }
}
