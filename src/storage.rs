use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::{fs, path::Path};
use chrono::Local;

use crate::task::Task;

/// Storage format version for future compatibility
const STORAGE_VERSION: u32 = 1;

/// Container for all tasks with metadata
#[derive(Debug, Serialize, Deserialize)]
pub struct TaskStore {
    pub version: u32,
    pub created_at: chrono::DateTime<Local>,
    pub modified_at: chrono::DateTime<Local>,
    pub tasks: Vec<Task>,
}

impl TaskStore {
    /// Create a new empty task store
    pub fn new() -> Self {
        let now = Local::now();
        Self {
            version: STORAGE_VERSION,
            created_at: now,
            modified_at: now,
            tasks: Vec::new(),
        }
    }

    /// Load tasks from a JSON file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let path = path.as_ref();
        
        if !path.exists() {
            return Ok(Self::new());
        }

        let content = fs::read_to_string(path)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        if content.trim().is_empty() {
            return Ok(Self::new());
        }

        // Try to parse as JSON first
        if let Ok(store) = serde_json::from_str::<TaskStore>(&content) {
            return Ok(store);
        }

        // If JSON parsing fails, try to parse as legacy text format
        Self::load_from_legacy_text(&content)
    }

    /// Load tasks from legacy text format (backward compatibility)
    fn load_from_legacy_text(content: &str) -> Result<Self> {
        let mut store = Self::new();
        
        let lines: Vec<String> = content
            .lines()
            .map(|line| line.trim().to_string())
            .filter(|line| !line.is_empty())
            .collect();

        for line in lines {
            if let Ok(task) = Task::from_string(&line) {
                store.tasks.push(task);
            }
        }

        Ok(store)
    }

    /// Save tasks to a JSON file
    pub fn save_to_file<P: AsRef<Path>>(&mut self, path: P) -> Result<()> {
        let path = path.as_ref();
        self.modified_at = Local::now();

        let json = serde_json::to_string_pretty(self)
            .context("Failed to serialize tasks to JSON")?;

        fs::write(path, json)
            .with_context(|| format!("Failed to write to file: {}", path.display()))?;

        Ok(())
    }

    /// Export tasks to legacy text format
    pub fn export_to_text(&self) -> String {
        self.tasks
            .iter()
            .map(|task| task.to_file_string())
            .collect::<Vec<_>>()
            .join("\n")
    }

    /// Export tasks to Taskwarrior JSON format
    pub fn export_to_taskwarrior(&self) -> Result<String> {
        let taskwarrior_tasks: Vec<TaskwarriorTask> = self.tasks
            .iter()
            .map(|task| TaskwarriorTask::from_task(task))
            .collect();

        serde_json::to_string_pretty(&taskwarrior_tasks)
            .context("Failed to serialize tasks to Taskwarrior format")
    }

    /// Import tasks from Taskwarrior JSON format
    pub fn import_from_taskwarrior(&mut self, json: &str) -> Result<()> {
        let taskwarrior_tasks: Vec<TaskwarriorTask> = serde_json::from_str(json)
            .context("Failed to parse Taskwarrior JSON")?;

        for tw_task in taskwarrior_tasks {
            if let Ok(task) = tw_task.to_task() {
                self.tasks.push(task);
            }
        }

        self.modified_at = Local::now();
        Ok(())
    }

    /// Add a task to the store
    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
        self.modified_at = Local::now();
    }

    /// Remove a task by UUID
    pub fn remove_task_by_id(&mut self, id: uuid::Uuid) -> Option<Task> {
        if let Some(pos) = self.tasks.iter().position(|t| t.id == id) {
            let task = self.tasks.remove(pos);
            self.modified_at = Local::now();
            Some(task)
        } else {
            None
        }
    }

    /// Get a task by UUID
    pub fn get_task_by_id(&self, id: uuid::Uuid) -> Option<&Task> {
        self.tasks.iter().find(|t| t.id == id)
    }

    /// Get a mutable task by UUID
    pub fn get_task_by_id_mut(&mut self, id: uuid::Uuid) -> Option<&mut Task> {
        self.tasks.iter_mut().find(|t| t.id == id)
    }

    /// Get all tasks
    pub fn get_tasks(&self) -> &[Task] {
        &self.tasks
    }

    /// Get all tasks (mutable)
    pub fn get_tasks_mut(&mut self) -> &mut Vec<Task> {
        &mut self.tasks
    }

    /// Clear all tasks
    pub fn clear(&mut self) {
        self.tasks.clear();
        self.modified_at = Local::now();
    }

    /// Get task count
    pub fn len(&self) -> usize {
        self.tasks.len()
    }

    /// Check if store is empty
    pub fn is_empty(&self) -> bool {
        self.tasks.is_empty()
    }
}

/// Taskwarrior-compatible task format
#[derive(Debug, Serialize, Deserialize)]
struct TaskwarriorTask {
    id: u32,
    description: String,
    status: String,
    uuid: String,
    entry: String,
    modified: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    due: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    priority: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    project: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    tags: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    depends: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    annotation: Option<String>,
}

impl TaskwarriorTask {
    /// Convert from our Task to Taskwarrior format
    fn from_task(task: &Task) -> Self {
        Self {
            id: 0, // Taskwarrior uses sequential IDs, we'll use 0 as placeholder
            description: task.description.clone(),
            status: match task.status {
                crate::task::TaskStatus::Pending => "pending".to_string(),
                crate::task::TaskStatus::InProgress => "in-progress".to_string(),
                crate::task::TaskStatus::Completed => "completed".to_string(),
                crate::task::TaskStatus::Deleted => "deleted".to_string(),
                crate::task::TaskStatus::Waiting => "waiting".to_string(),
            },
            uuid: task.id.to_string(),
            entry: task.created_at.format("%Y%m%dT%H%M%SZ").to_string(),
            modified: task.modified_at.format("%Y%m%dT%H%M%SZ").to_string(),
            due: task.due_date.map(|d| d.format("%Y%m%dT%H%M%SZ").to_string()),
            priority: match task.priority {
                crate::task::Priority::High => Some("H".to_string()),
                crate::task::Priority::Medium => Some("M".to_string()),
                crate::task::Priority::Low => Some("L".to_string()),
                crate::task::Priority::None => None,
            },
            project: task.project.clone(),
            tags: task.tags.clone(),
            depends: task.depends_on.iter().map(|id| id.to_string()).collect(),
            annotation: task.notes.clone(),
        }
    }

    /// Convert from Taskwarrior format to our Task
    fn to_task(&self) -> Result<Task> {
        let id = uuid::Uuid::parse_str(&self.uuid)
            .context("Invalid UUID in Taskwarrior task")?;

        let created_at = chrono::DateTime::parse_from_rfc3339(&self.entry)
            .context("Invalid entry date in Taskwarrior task")?
            .with_timezone(&chrono::Local);

        let modified_at = chrono::DateTime::parse_from_rfc3339(&self.modified)
            .context("Invalid modified date in Taskwarrior task")?
            .with_timezone(&chrono::Local);

        let due_date = if let Some(due_str) = &self.due {
            Some(
                chrono::DateTime::parse_from_rfc3339(due_str)
                    .context("Invalid due date in Taskwarrior task")?
                    .with_timezone(&chrono::Local)
            )
        } else {
            None
        };

        let priority = match self.priority.as_deref() {
            Some("H") => crate::task::Priority::High,
            Some("M") => crate::task::Priority::Medium,
            Some("L") => crate::task::Priority::Low,
            _ => crate::task::Priority::None,
        };

        let status = match self.status.as_str() {
            "pending" => crate::task::TaskStatus::Pending,
            "in-progress" => crate::task::TaskStatus::InProgress,
            "completed" => crate::task::TaskStatus::Completed,
            "deleted" => crate::task::TaskStatus::Deleted,
            "waiting" => crate::task::TaskStatus::Waiting,
            _ => crate::task::TaskStatus::Pending,
        };

        let depends_on = self.depends
            .iter()
            .filter_map(|dep_id| uuid::Uuid::parse_str(dep_id).ok())
            .collect();

        Ok(Task {
            id,
            description: self.description.clone(),
            status,
            priority,
            created_at,
            modified_at,
            due_date,
            recurrence: None, // Taskwarrior doesn't have direct recurrence support
            project: self.project.clone(),
            tags: self.tags.clone(),
            depends_on,
            notes: self.annotation.clone(),
            estimated_duration: None, // Not supported in Taskwarrior
        })
    }
}
