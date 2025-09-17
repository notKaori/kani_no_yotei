use anyhow::Result;
use chrono::{DateTime, Local, Duration};
use serde::{Deserialize, Serialize};
use notify_rust::Notification;

/// A task with optional due date/time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub text: String,
    pub due_date: Option<DateTime<Local>>,
}

impl Task {
    /// Create a new task with just text
    pub fn new(text: String) -> Self {
        Self {
            text,
            due_date: None,
        }
    }

    /// Create a new task with text and due date
    pub fn with_due_date(text: String, due_date: DateTime<Local>) -> Self {
        Self {
            text,
            due_date: Some(due_date),
        }
    }

    /// Parse a task from a string, supporting formats like:
    /// "task text" or "task text @ 2024-01-15 14:30" or "task text @ tomorrow 9am"
    pub fn from_string(s: &str) -> Result<Self> {
        // Look for @ symbol to indicate due date
        if let Some(at_pos) = s.find(" @ ") {
            let text = s[..at_pos].trim().to_string();
            let date_str = s[at_pos + 3..].trim();
            
            if let Ok(due_date) = crate::date_utils::parse_due_date(date_str) {
                Ok(Self::with_due_date(text, due_date))
            } else {
                // If date parsing fails, treat the whole thing as text
                Ok(Self::new(s.to_string()))
            }
        } else {
            Ok(Self::new(s.to_string()))
        }
    }

    /// Convert task to string for display
    pub fn to_display_string(&self) -> String {
        if let Some(due_date) = &self.due_date {
            format!("{} @ {}", self.text, due_date.format("%Y-%m-%d %H:%M"))
        } else {
            self.text.clone()
        }
    }

    /// Convert task to string for file storage
    pub fn to_file_string(&self) -> String {
        if let Some(due_date) = &self.due_date {
            format!("{} @ {}", self.text, due_date.format("%Y-%m-%d %H:%M"))
        } else {
            self.text.clone()
        }
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
            format!("{}\nDue: {}", self.text, due_date.format("%Y-%m-%d %H:%M"))
        } else {
            self.text.clone()
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
pub fn print_tasks(tasks: &[Task]) {
    for (i, task) in tasks.iter().enumerate() {
        println!("{}. {}", i + 1, task.to_display_string());
    }
}

/// Remove a task by index (1-based) or by text match
/// Returns the removed task text and whether removal was successful
pub fn remove_task(tasks: &mut Vec<Task>, input: &str) -> (String, bool) {
    // First, try to parse as an index
    if let Ok(index) = input.parse::<usize>() {
        if index > 0 && index <= tasks.len() {
            let removed_task = tasks.remove(index - 1);
            return (removed_task.text, true);
        } else {
            return (format!("Invalid index: {}. Valid range: 1-{}", index, tasks.len()), false);
        }
    }
    
    // If not a valid index, try text matching
    let initial_len = tasks.len();
    tasks.retain(|t| t.text != input);
    let removed_count = initial_len - tasks.len();
    
    if removed_count > 0 {
        (input.to_string(), true)
    } else {
        (format!("Task not found: {}", input), false)
    }
}
