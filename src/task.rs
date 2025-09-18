use anyhow::Result;
use chrono::{DateTime, Local, Duration};
use serde::{Deserialize, Serialize};
use notify_rust::Notification;
use uuid::Uuid;

/// Task status enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Pending,
    InProgress,
    Completed,
    Deleted,
    Waiting,
}

/// Task priority enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Priority {
    High,
    Medium,
    Low,
    None,
}

/// Recurrence rule for repeating tasks
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RecurrenceRule {
    pub frequency: RecurrenceFrequency,
    pub interval: u32, // Every N days/weeks/months
    pub end_date: Option<DateTime<Local>>,
    pub count: Option<u32>, // Number of occurrences
}

/// Recurrence frequency types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RecurrenceFrequency {
    Daily,
    Weekly,
    Monthly,
    Yearly,
}

/// Enhanced task structure with scheduling capabilities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    // Core fields
    pub id: Uuid,
    pub description: String,
    pub status: TaskStatus,
    pub priority: Priority,
    pub created_at: DateTime<Local>,
    pub modified_at: DateTime<Local>,
    
    // Scheduling fields
    pub due_date: Option<DateTime<Local>>,
    pub recurrence: Option<RecurrenceRule>,
    
    // Organization fields
    pub project: Option<String>,
    pub tags: Vec<String>,
    
    // Dependencies
    pub depends_on: Vec<Uuid>,
    
    // Additional metadata
    pub notes: Option<String>,
    pub estimated_duration: Option<Duration>,
}

impl Task {
    /// Create a new task with just description
    pub fn new(description: String) -> Self {
        let now = Local::now();
        Self {
            id: Uuid::new_v4(),
            description,
            status: TaskStatus::Pending,
            priority: Priority::None,
            created_at: now,
            modified_at: now,
            due_date: None,
            recurrence: None,
            project: None,
            tags: Vec::new(),
            depends_on: Vec::new(),
            notes: None,
            estimated_duration: None,
        }
    }

    /// Create a new task with description and due date
    pub fn with_due_date(description: String, due_date: DateTime<Local>) -> Self {
        let now = Local::now();
        Self {
            id: Uuid::new_v4(),
            description,
            status: TaskStatus::Pending,
            priority: Priority::None,
            created_at: now,
            modified_at: now,
            due_date: Some(due_date),
            recurrence: None,
            project: None,
            tags: Vec::new(),
            depends_on: Vec::new(),
            notes: None,
            estimated_duration: None,
        }
    }

    /// Create a new task with full specification
    pub fn new_full(
        description: String,
        priority: Priority,
        due_date: Option<DateTime<Local>>,
        project: Option<String>,
        tags: Vec<String>,
    ) -> Self {
        let now = Local::now();
        Self {
            id: Uuid::new_v4(),
            description,
            status: TaskStatus::Pending,
            priority,
            created_at: now,
            modified_at: now,
            due_date,
            recurrence: None,
            project,
            tags,
            depends_on: Vec::new(),
            notes: None,
            estimated_duration: None,
        }
    }

    /// Parse a task from a string, supporting formats like:
    /// "task text" or "task text @ 2024-01-15 14:30" or "task text @ tomorrow 9am"
    /// Also supports legacy format for backward compatibility
    pub fn from_string(s: &str) -> Result<Self> {
        // Look for @ symbol to indicate due date
        if let Some(at_pos) = s.find(" @ ") {
            let description = s[..at_pos].trim().to_string();
            let date_str = s[at_pos + 3..].trim();
            
            if let Ok(due_date) = crate::date_utils::parse_due_date(date_str) {
                Ok(Self::with_due_date(description, due_date))
            } else {
                // If date parsing fails, treat the whole thing as description
                Ok(Self::new(s.to_string()))
            }
        } else {
            Ok(Self::new(s.to_string()))
        }
    }

    /// Update the modified timestamp
    pub fn touch(&mut self) {
        self.modified_at = Local::now();
    }

    /// Mark task as completed
    pub fn complete(&mut self) {
        self.status = TaskStatus::Completed;
        self.touch();
    }

    /// Mark task as in progress
    pub fn start(&mut self) {
        self.status = TaskStatus::InProgress;
        self.touch();
    }

    /// Add a tag to the task
    pub fn add_tag(&mut self, tag: String) {
        if !self.tags.contains(&tag) {
            self.tags.push(tag);
            self.touch();
        }
    }

    /// Remove a tag from the task
    pub fn remove_tag(&mut self, tag: &str) {
        self.tags.retain(|t| t != tag);
        self.touch();
    }

    /// Set the project for the task
    pub fn set_project(&mut self, project: Option<String>) {
        self.project = project;
        self.touch();
    }

    /// Set the priority for the task
    pub fn set_priority(&mut self, priority: Priority) {
        self.priority = priority;
        self.touch();
    }

    /// Convert task to string for display
    pub fn to_display_string(&self) -> String {
        let mut result = String::new();
        
        // Add status indicator
        match self.status {
            TaskStatus::Completed => result.push_str("✓ "),
            TaskStatus::InProgress => result.push_str("▶ "),
            TaskStatus::Waiting => result.push_str("⏸ "),
            TaskStatus::Deleted => result.push_str("✗ "),
            TaskStatus::Pending => result.push_str("○ "),
        }
        
        // Add priority indicator
        match self.priority {
            Priority::High => result.push_str("H"),
            Priority::Medium => result.push_str("M"),
            Priority::Low => result.push_str("L"),
            Priority::None => {},
        }
        
        // Add description
        result.push_str(&self.description);
        
        // Add due date
        if let Some(due_date) = &self.due_date {
            result.push_str(&format!(" @ {}", due_date.format("%Y-%m-%d %H:%M")));
        }
        
        // Add project
        if let Some(project) = &self.project {
            result.push_str(&format!(" [{}]", project));
        }
        
        // Add tags
        if !self.tags.is_empty() {
            result.push_str(&format!(" #{}", self.tags.join(" #")));
        }
        
        result
    }

    /// Convert task to string for file storage (legacy format for backward compatibility)
    pub fn to_file_string(&self) -> String {
        if let Some(due_date) = &self.due_date {
            format!("{} @ {}", self.description, due_date.format("%Y-%m-%d %H:%M"))
        } else {
            self.description.clone()
        }
    }

    /// Convert task to detailed string for verbose display
    pub fn to_verbose_string(&self) -> String {
        let mut result = String::new();
        
        result.push_str(&format!("ID: {}\n", self.id));
        result.push_str(&format!("Description: {}\n", self.description));
        result.push_str(&format!("Status: {:?}\n", self.status));
        result.push_str(&format!("Priority: {:?}\n", self.priority));
        result.push_str(&format!("Created: {}\n", self.created_at.format("%Y-%m-%d %H:%M:%S")));
        result.push_str(&format!("Modified: {}\n", self.modified_at.format("%Y-%m-%d %H:%M:%S")));
        
        if let Some(due_date) = &self.due_date {
            result.push_str(&format!("Due: {}\n", due_date.format("%Y-%m-%d %H:%M:%S")));
        }
        
        if let Some(project) = &self.project {
            result.push_str(&format!("Project: {}\n", project));
        }
        
        if !self.tags.is_empty() {
            result.push_str(&format!("Tags: {}\n", self.tags.join(", ")));
        }
        
        if !self.depends_on.is_empty() {
            result.push_str(&format!("Depends on: {}\n", 
                self.depends_on.iter().map(|id| id.to_string()).collect::<Vec<_>>().join(", ")));
        }
        
        if let Some(notes) = &self.notes {
            result.push_str(&format!("Notes: {}\n", notes));
        }
        
        if let Some(duration) = &self.estimated_duration {
            result.push_str(&format!("Estimated duration: {} minutes\n", duration.num_minutes()));
        }
        
        result
    }

    /// Check if this task is due within the specified minutes
    pub fn is_due_soon(&self, minutes_before: u32) -> bool {
        if let Some(due_date) = &self.due_date {
            let now = Local::now();
            let notification_time = *due_date - Duration::minutes(minutes_before as i64);
            now >= notification_time && now <= *due_date
        } else {
            false
        }
    }

    /// Check if this task is overdue
    pub fn is_overdue(&self) -> bool {
        if let Some(due_date) = &self.due_date {
            Local::now() > *due_date
        } else {
            false
        }
    }

    /// Send a notification for this task
    pub fn send_notification(&self, is_overdue: bool) -> Result<()> {
        let title = if is_overdue {
            "⚠️ Task Overdue"
        } else {
            "📅 Task Due Soon"
        };

        let body = if let Some(due_date) = &self.due_date {
            format!("{}\nDue: {}", self.description, due_date.format("%Y-%m-%d %H:%M"))
        } else {
            self.description.clone()
        };

        // Try desktop notification first
        match Notification::new()
            .summary(title)
            .body(&body)
            .icon("task-due")
            .timeout(10000) // 10 seconds
            .show()
        {
            Ok(_) => Ok(()),
            Err(_) => {
                // Fallback to terminal notification
                println!("🔔 {}: {}", title, body);
                Ok(())
            }
        }
    }
}

/// Prints all tasks in the todo list
pub fn print_tasks(tasks: &[Task], verbose: bool) {
    if verbose {
        println!("Printing {} tasks:", tasks.len());
    }
    for (i, task) in tasks.iter().enumerate() {
        println!("{}. {}", i + 1, task.to_display_string());
    }
}

/// Remove a task by index (1-based), UUID, or by description match
/// Returns the removed task description and whether removal was successful
pub fn remove_task(tasks: &mut Vec<Task>, input: &str) -> (String, bool) {
    // First, try to parse as an index
    if let Ok(index) = input.parse::<usize>() {
        if index > 0 && index <= tasks.len() {
            let removed_task = tasks.remove(index - 1);
            return (removed_task.description, true);
        } else {
            return (format!("Invalid index: {}. Valid range: 1-{}", index, tasks.len()), false);
        }
    }
    
    // Try to parse as UUID
    if let Ok(uuid) = Uuid::parse_str(input) {
        if let Some(pos) = tasks.iter().position(|t| t.id == uuid) {
            let removed_task = tasks.remove(pos);
            return (removed_task.description, true);
        } else {
            return (format!("Task with UUID {} not found", uuid), false);
        }
    }
    
    // If not a valid index or UUID, try description matching
    let initial_len = tasks.len();
    tasks.retain(|t| t.description != input);
    let removed_count = initial_len - tasks.len();
    
    if removed_count > 0 {
        (input.to_string(), true)
    } else {
        (format!("Task not found: {}", input), false)
    }
}

/// Find a task by UUID
pub fn find_task_by_id(tasks: &[Task], id: Uuid) -> Option<&Task> {
    tasks.iter().find(|t| t.id == id)
}

/// Find a task by UUID (mutable)
pub fn find_task_by_id_mut(tasks: &mut [Task], id: Uuid) -> Option<&mut Task> {
    tasks.iter_mut().find(|t| t.id == id)
}

/// Filter tasks by status
pub fn filter_tasks_by_status(tasks: &[Task], status: TaskStatus) -> Vec<&Task> {
    tasks.iter().filter(|t| t.status == status).collect()
}

/// Filter tasks by priority
pub fn filter_tasks_by_priority(tasks: &[Task], priority: Priority) -> Vec<&Task> {
    tasks.iter().filter(|t| t.priority == priority).collect()
}

/// Filter tasks by project
pub fn filter_tasks_by_project<'a>(tasks: &'a [Task], project: &str) -> Vec<&'a Task> {
    tasks.iter().filter(|t| t.project.as_ref().map_or(false, |p| p == project)).collect()
}

/// Filter tasks by tag
pub fn filter_tasks_by_tag<'a>(tasks: &'a [Task], tag: &str) -> Vec<&'a Task> {
    tasks.iter().filter(|t| t.tags.contains(&tag.to_string())).collect()
}
