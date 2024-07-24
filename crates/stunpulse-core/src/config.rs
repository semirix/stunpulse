/// Configuration settings for the application.
#[derive(Default)]
pub struct Configuration {
    pub layout: TaskTableLayout,
}

/// Represents the layout of a task table with various attributes.
pub struct TaskTableLayout {
    pub table: String,
    pub id: String,
    pub version: String,
    pub name: String,
    pub parameters: String,
    pub metadata: String,
}

impl Default for TaskTableLayout {
    fn default() -> Self {
        Self {
            table: "tasks".into(),
            id: "task_id".into(),
            version: "task_version".into(),
            name: "task_name".into(),
            parameters: "task_parameters".into(),
            metadata: "task_metadata".into(),
        }
    }
}
