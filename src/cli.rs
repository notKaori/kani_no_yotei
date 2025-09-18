use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::{fs, io::{self, Write}, path::{Path, PathBuf}, process::Command};
use chrono::Local;

use crate::task::{Task, print_tasks, remove_task};
use crate::storage::TaskStore;
use crate::utils::LockGuard;

/// Kani No Yotei - A simple todo list manager
#[derive(Parser)]
#[command(name = "kny")]
#[command(about = "A simple todo list manager")]
#[command(version)]
pub struct Cli {
    /// Enable debug mode (shows flag information)
    #[arg(short, long, global = true)]
    pub debug: bool,
    
    #[command(subcommand)]
    pub command: Option<Commands>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Add one or more tasks to the list
    Add {
        /// Tasks to add (quotes required for multi-word tasks)
        tasks: Vec<String>,
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
        /// Force adding even if tasks exist
        #[arg(short, long)]
        force: bool,
    },
    /// Print all current tasks
    Print {
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
        /// Sync flag
        #[arg(short = 'S')]
        sync: bool,
        /// Yes flag
        #[arg(short = 'y')]
        yes: bool,
    },
    /// Save tasks to file
    Save {
        /// File to save to (default: ToDo.txt)
        file: Option<String>,
    },
    /// Open a different todo file
    Open {
        /// File to open
        file: String,
    },
    /// Remove a task from the list
    Remove {
        /// Task to remove (by index number or text)
        task: String,
    },
    /// Clear all tasks from the list
    Clear,
    /// Show current working file
    Which,
    /// Check for due tasks and send notifications
    Notify {
        /// Minutes before due date to send notification (default: 15)
        #[arg(short, long, default_value = "15")]
        minutes_before: u32,
        /// Verbose output
        #[arg(short, long)]
        verbose: bool,
    },
    /// Setup automatic notifications with cron jobs
    Setup,
    /// Set task priority (high, medium, low, none)
    Priority {
        /// Task to modify (by index, UUID, or description)
        task: String,
        /// Priority level (high, medium, low, none)
        priority: String,
    },
    /// Set task status (pending, in-progress, completed, waiting, deleted)
    Status {
        /// Task to modify (by index, UUID, or description)
        task: String,
        /// Status (pending, in-progress, completed, waiting, deleted)
        status: String,
    },
    /// Set task project
    Project {
        /// Task to modify (by index, UUID, or description)
        task: String,
        /// Project name
        project: String,
    },
    /// Add tag to task
    Tag {
        /// Task to modify (by index, UUID, or description)
        task: String,
        /// Tag to add
        tag: String,
    },
    /// Remove tag from task
    Untag {
        /// Task to modify (by index, UUID, or description)
        task: String,
        /// Tag to remove
        tag: String,
    },
    /// Export tasks to text format
    Export {
        /// Output file (default: stdout)
        file: Option<String>,
        /// Export format (text, taskwarrior)
        #[arg(short, long, default_value = "text")]
        format: String,
    },
    /// Import tasks from Taskwarrior JSON
    Import {
        /// Input file
        file: String,
    },
}

/// Execute a clap command with proper error handling
pub fn execute_clap_command(cmd: Commands, store: &mut TaskStore, filepath: &mut PathBuf, auto_save: bool, debug: bool) -> Result<()> {
    match cmd {
        Commands::Add { tasks, verbose, force } => {
            if tasks.is_empty() {
                println!("Error: No tasks provided");
                println!("Usage: kny add \"<task>\" [\"<task2>\"] [flags]");
                println!("       kny add \"<task> @ <date/time>\" for tasks with due dates");
                println!("       Note: The entire task including @ must be in quotes");
                println!("       Example: kny add \"do laundry @ tomorrow\"");
                return Ok(());
            }
            
            let mut added_count = 0;
            for (i, task_str) in tasks.iter().enumerate() {
                if debug && verbose {
                    println!("Adding task {} at index '{}'", task_str, i + 1);
                }
                
                let task = Task::from_string(task_str)?;
                store.add_task(task);
                added_count += 1;
            }
            
            if debug && force {
                println!("Force: Adding {} tasks even if they exist", added_count);
            }
            
            if added_count == 1 {
                println!("You'd like to add {}", tasks[0]);
            } else {
                println!("Added {} tasks to the list", added_count);
            }
            
            if auto_save {
                let path_display = filepath.display().to_string();
                store.save_to_file(filepath)?;
                if added_count == 1 {
                    println!("Task added and saved to {:?}", path_display);
                } else {
                    println!("Tasks added and saved to {:?}", path_display);
                }
            }
        }
        Commands::Print { verbose, sync, yes } => {
            if debug {
                if verbose {
                    println!("Printing with verbose flag");
                }
                if sync {
                    println!("Printing with sync flag");
                }
                if yes {
                    println!("Printing with yes flag");
                }
            }
            
            if store.is_empty() {
                println!("No tasks to print");
            } else {
                print_tasks(store.get_tasks(), verbose);
            }
        }
        Commands::Save { file } => {
            if debug {
                if let Some(file_path) = &file {
                    println!("Saving to custom file: {}", file_path);
                } else {
                    println!("Saving to default file");
                }
            }
            
            let target_path = if let Some(file_path) = file {
                PathBuf::from(&file_path)
            } else {
                filepath.clone()
            };
            
            if store.is_empty() {
                println!("Please enter at least 1 task before saving");
                return Ok(());
            }
            
            store.save_to_file(&target_path)?;
            println!("saving to {:?}", target_path.display());
        }
        Commands::Open { file } => {
            if debug {
                println!("Opening file: {}", file);
            }
            
            *filepath = PathBuf::from(&file);
            *store = TaskStore::load_from_file(&filepath)?;
            println!("Opening file: {}", file);
            print_tasks(store.get_tasks(), false);
        }
        Commands::Remove { task } => {
            if debug {
                println!("Removing task: {}", task);
            }
            
            let (removed_text, success) = remove_task(store.get_tasks_mut(), &task);
            
            if success {
                println!("Removed task: {}", removed_text);
                
                if auto_save {
                    let path_display = filepath.display().to_string();
                    store.save_to_file(filepath)?;
                    println!("Task removed and saved to {:?}", path_display);
                }
            } else {
                println!("{}", removed_text);
            }
        }
        Commands::Clear => {
            if debug {
                println!("Clearing all tasks from the list");
            }
            
            store.clear();
            println!("Cleared task list");
            
            if auto_save {
                let path_display = filepath.display().to_string();
                store.save_to_file(filepath)?;
                println!("Task list cleared and saved to {:?}", path_display);
            }
        }
        Commands::Which => {
            if debug {
                println!("Showing current working file");
            }
            println!("Current file: {}", filepath.display());
        }
        Commands::Notify { minutes_before, verbose } => {
            if debug {
                println!("Checking for tasks due within {} minutes", minutes_before);
                if verbose {
                    println!("Running in verbose mode");
                }
            }
            
            // Create lock file to prevent overlapping cron jobs
            let lock_file = filepath.with_extension("lock");
            if lock_file.exists() {
                if verbose {
                    println!("Lock file exists, skipping notification check");
                }
                return Ok(());
            }
            
            // Create lock file
            fs::write(&lock_file, format!("{}", Local::now().timestamp()))
                .context("Failed to create lock file")?;
            
            // Ensure lock file is removed when function exits
            let _guard = LockGuard::new(lock_file);
            
            if verbose {
                println!("Checking for tasks due within {} minutes", minutes_before);
            }
            
            let mut notifications_sent = 0;
            let mut overdue_tasks = Vec::new();
            let mut due_soon_tasks = Vec::new();
            
            // Check for overdue tasks first
            for task in store.get_tasks() {
                if task.is_overdue() {
                    overdue_tasks.push(task);
                } else if task.is_due_soon(minutes_before) {
                    due_soon_tasks.push(task);
                }
            }
            
            // Send notifications for overdue tasks
            for task in overdue_tasks {
                if verbose {
                    println!("Sending overdue notification for: {}", task.description);
                }
                task.send_notification(true)?;
                notifications_sent += 1;
            }
            
            // Send notifications for tasks due soon
            for task in due_soon_tasks {
                if verbose {
                    println!("Sending due soon notification for: {}", task.description);
                }
                task.send_notification(false)?;
                notifications_sent += 1;
            }
            
            if notifications_sent == 0 {
                if verbose {
                    println!("No tasks due within {} minutes", minutes_before);
                }
            } else {
                println!("Sent {} notifications", notifications_sent);
            }
        }
        Commands::Setup => {
            if debug {
                println!("Starting notification setup");
            }
            setup_notifications(filepath)?;
        }
        Commands::Priority { task, priority } => {
            if debug {
                println!("Setting priority for task: {} to {}", task, priority);
            }
            
            let priority_enum = match priority.to_lowercase().as_str() {
                "high" | "h" => crate::task::Priority::High,
                "medium" | "m" => crate::task::Priority::Medium,
                "low" | "l" => crate::task::Priority::Low,
                "none" | "n" => crate::task::Priority::None,
                _ => {
                    println!("Invalid priority: {}. Use: high, medium, low, none", priority);
                    return Ok(());
                }
            };
            
            if let Some(task_ref) = find_task_by_identifier(store, &task) {
                if let Some(task_mut) = store.get_task_by_id_mut(task_ref.id) {
                    task_mut.set_priority(priority_enum);
                    println!("Set priority to {} for task: {}", priority, task_mut.description);
                    
                    if auto_save {
                        store.save_to_file(filepath)?;
                    }
                }
            } else {
                println!("Task not found: {}", task);
            }
        }
        Commands::Status { task, status } => {
            if debug {
                println!("Setting status for task: {} to {}", task, status);
            }
            
            let status_enum = match status.to_lowercase().as_str() {
                "pending" | "p" => crate::task::TaskStatus::Pending,
                "in-progress" | "inprogress" | "i" => crate::task::TaskStatus::InProgress,
                "completed" | "done" | "c" => crate::task::TaskStatus::Completed,
                "waiting" | "w" => crate::task::TaskStatus::Waiting,
                "deleted" | "d" => crate::task::TaskStatus::Deleted,
                _ => {
                    println!("Invalid status: {}. Use: pending, in-progress, completed, waiting, deleted", status);
                    return Ok(());
                }
            };
            
            if let Some(task_ref) = find_task_by_identifier(store, &task) {
                if let Some(task_mut) = store.get_task_by_id_mut(task_ref.id) {
                    task_mut.status = status_enum;
                    task_mut.touch();
                    println!("Set status to {} for task: {}", status, task_mut.description);
                    
                    if auto_save {
                        store.save_to_file(filepath)?;
                    }
                }
            } else {
                println!("Task not found: {}", task);
            }
        }
        Commands::Project { task, project } => {
            if debug {
                println!("Setting project for task: {} to {}", task, project);
            }
            
            if let Some(task_ref) = find_task_by_identifier(store, &task) {
                if let Some(task_mut) = store.get_task_by_id_mut(task_ref.id) {
                    task_mut.set_project(Some(project.clone()));
                    println!("Set project to {} for task: {}", project, task_mut.description);
                    
                    if auto_save {
                        store.save_to_file(filepath)?;
                    }
                }
            } else {
                println!("Task not found: {}", task);
            }
        }
        Commands::Tag { task, tag } => {
            if debug {
                println!("Adding tag {} to task: {}", tag, task);
            }
            
            if let Some(task_ref) = find_task_by_identifier(store, &task) {
                if let Some(task_mut) = store.get_task_by_id_mut(task_ref.id) {
                    task_mut.add_tag(tag.clone());
                    println!("Added tag {} to task: {}", tag, task_mut.description);
                    
                    if auto_save {
                        store.save_to_file(filepath)?;
                    }
                }
            } else {
                println!("Task not found: {}", task);
            }
        }
        Commands::Untag { task, tag } => {
            if debug {
                println!("Removing tag {} from task: {}", tag, task);
            }
            
            if let Some(task_ref) = find_task_by_identifier(store, &task) {
                if let Some(task_mut) = store.get_task_by_id_mut(task_ref.id) {
                    task_mut.remove_tag(&tag);
                    println!("Removed tag {} from task: {}", tag, task_mut.description);
                    
                    if auto_save {
                        store.save_to_file(filepath)?;
                    }
                }
            } else {
                println!("Task not found: {}", task);
            }
        }
        Commands::Export { file, format } => {
            if debug {
                println!("Exporting tasks in {} format", format);
            }
            
            let content = match format.to_lowercase().as_str() {
                "text" => store.export_to_text(),
                "taskwarrior" => store.export_to_taskwarrior()?,
                _ => {
                    println!("Invalid format: {}. Use: text, taskwarrior", format);
                    return Ok(());
                }
            };
            
            if let Some(file_path) = file {
                fs::write(&file_path, content)
                    .with_context(|| format!("Failed to write to {}", file_path))?;
                println!("Exported {} tasks to {}", store.len(), file_path);
            } else {
                print!("{}", content);
            }
        }
        Commands::Import { file } => {
            if debug {
                println!("Importing tasks from {}", file);
            }
            
            let content = fs::read_to_string(&file)
                .with_context(|| format!("Failed to read file: {}", file))?;
            
            store.import_from_taskwarrior(&content)?;
            println!("Imported tasks from {}", file);
            
            if auto_save {
                store.save_to_file(filepath)?;
            }
        }
    }
    Ok(())
}

/// Find a task by identifier (index, UUID, or description)
fn find_task_by_identifier<'a>(store: &'a TaskStore, identifier: &str) -> Option<&'a Task> {
    // Try to parse as index
    if let Ok(index) = identifier.parse::<usize>() {
        if index > 0 && index <= store.len() {
            return store.get_tasks().get(index - 1);
        }
    }
    
    // Try to parse as UUID
    if let Ok(uuid) = uuid::Uuid::parse_str(identifier) {
        return store.get_task_by_id(uuid);
    }
    
    // Try to find by description
    store.get_tasks().iter().find(|t| t.description == identifier)
}

/// Setup automatic notifications with cron jobs
fn setup_notifications(filpath: &PathBuf) -> Result<()> {
    println!("🦀 Kani No Yotei - Notification Setup");
    println!("==================================");
    println!();

    // Get the absolute path to the kny binary
    let kny_binary = std::env::current_exe()
        .context("Failed to get current executable path")?;
    
    println!("✅ Found kny binary: {}", kny_binary.display());
    println!();

    // Get current working directory for todo file
    let todo_dir = filpath.parent()
        .context("Failed to get todo file directory")?;
    let todo_file = todo_dir.join("tasks.json");

    println!("📁 Todo file location: {}", todo_file.display());
    println!();

    // Check if todo file exists
    if !todo_file.exists() {
        println!("⚠️  Warning: tasks.json not found in current directory");
        println!("The notification system will work, but you may want to create some tasks first.");
        println!();
    }

    // Main menu
    loop {
        println!("Choose an option:");
        println!("1) Add notification check every 5 minutes (15 min advance warning)");
        println!("2) Add notification check every 10 minutes (15 min advance warning)");
        println!("3) Add notification check every 15 minutes (15 min advance warning)");
        println!("4) Add custom notification schedule");
        println!("5) Show current notification jobs");
        println!("6) Remove all notification jobs");
        println!("7) Test notification system");
        println!("8) Exit");
        println!();
        
        print!("Enter your choice (1-8): ");
        io::stdout().flush()?;
        
        let mut input = String::new();
        io::stdin().read_line(&mut input)?;
        let choice = input.trim();

        match choice {
            "1" => add_cron_job("*/5 * * * *", "15", todo_dir, &kny_binary)?,
            "2" => add_cron_job("*/10 * * * *", "15", todo_dir, &kny_binary)?,
            "3" => add_cron_job("*/15 * * * *", "15", todo_dir, &kny_binary)?,
            "4" => {
                println!();
                println!("Custom Schedule Setup");
                print!("Enter cron schedule (e.g., '*/5 * * * *' for every 5 minutes): ");
                io::stdout().flush()?;
                let mut schedule = String::new();
                io::stdin().read_line(&mut schedule)?;
                let schedule = schedule.trim();

                print!("Enter minutes before due date to send notification (default: 15): ");
                io::stdout().flush()?;
                let mut minutes = String::new();
                io::stdin().read_line(&mut minutes)?;
                let minutes = minutes.trim();
                let minutes = if minutes.is_empty() { "15" } else { minutes };

                add_cron_job(schedule, minutes, todo_dir, &kny_binary)?;
            }
            "5" => show_cron_jobs()?,
            "6" => remove_cron_jobs()?,
            "7" => {
                println!("🧪 Testing notification system...");
                let output = Command::new(&kny_binary)
                    .arg("notify")
                    .arg("-v")
                    .current_dir(todo_dir)
                    .output()?;
                
                if output.status.success() {
                    print!("{}", String::from_utf8_lossy(&output.stdout));
                } else {
                    eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                }
                println!();
            }
            "8" => {
                println!("👋 Setup complete!");
                println!();
                println!("💡 Tips:");
                println!("• Use 'kny notify -v' to test notifications manually");
                println!("• Use 'kny notify -m 30' to check for tasks due within 30 minutes");
                println!("• Use 'crontab -l' to view all your cron jobs");
                println!("• Use 'crontab -e' to manually edit cron jobs");
                println!();
                break;
            }
            _ => {
                println!("❌ Invalid choice. Please enter 1-8.");
                println!();
            }
        }
    }

    Ok(())
}

/// Add a cron job for notifications
fn add_cron_job(interval: &str, minutes_before: &str, todo_dir: &Path, kny_binary: &PathBuf) -> Result<()> {
    let cron_cmd = format!("cd {} && {} notify -m {}", todo_dir.display(), kny_binary.display(), minutes_before);
    
    // Get current crontab
    let output = Command::new("crontab")
        .arg("-l")
        .output()?;
    
    let mut crontab_content = if output.status.success() {
        String::from_utf8_lossy(&output.stdout).to_string()
    } else {
        String::new()
    };
    
    // Add new cron job
    crontab_content.push_str(&format!("{} {}\n", interval, cron_cmd));
    
    // Write new crontab
    let mut child = Command::new("crontab")
        .stdin(std::process::Stdio::piped())
        .spawn()?;
    
    child.stdin.as_mut().unwrap().write_all(crontab_content.as_bytes())?;
    child.wait()?;
    
    println!("✅ Added cron job: Check every {} for tasks due within {} minutes", interval, minutes_before);
    println!();
    
    Ok(())
}

/// Show current cron jobs for kny
fn show_cron_jobs() -> Result<()> {
    println!("📋 Current cron jobs for kny:");
    
    let output = Command::new("crontab")
        .arg("-l")
        .output()?;
    
    if output.status.success() {
        let crontab = String::from_utf8_lossy(&output.stdout);
        let kny_jobs: Vec<&str> = crontab.lines()
            .filter(|line| line.contains("kny notify"))
            .collect();
        
        if kny_jobs.is_empty() {
            println!("No kny notification jobs found.");
        } else {
            for job in kny_jobs {
                println!("{}", job);
            }
        }
    } else {
        println!("No kny notification jobs found.");
    }
    println!();
    
    Ok(())
}

/// Remove all kny cron jobs
fn remove_cron_jobs() -> Result<()> {
    println!("🗑️  Removing all kny notification cron jobs...");
    
    let output = Command::new("crontab")
        .arg("-l")
        .output()?;
    
    if output.status.success() {
        let crontab = String::from_utf8_lossy(&output.stdout);
        let filtered_lines: Vec<&str> = crontab.lines()
            .filter(|line| !line.contains("kny notify"))
            .collect();
        
        let new_crontab = filtered_lines.join("\n");
        
        let mut child = Command::new("crontab")
            .stdin(std::process::Stdio::piped())
            .spawn()?;
        
        child.stdin.as_mut().unwrap().write_all(new_crontab.as_bytes())?;
        child.wait()?;
    }
    
    println!("✅ Removed all kny notification cron jobs");
    println!();
    
    Ok(())
}
