use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use std::{fs, fs::File, io::BufRead, io::BufReader, path::PathBuf};
use chrono::Local;

use crate::task::{Task, print_tasks, remove_task};
use crate::utils::LockGuard;

/// Kani No Yotei - A simple todo list manager
#[derive(Parser)]
#[command(name = "kny")]
#[command(about = "A simple todo list manager")]
#[command(version)]
pub struct Cli {
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
}

/// Execute a clap command with proper error handling
pub fn execute_clap_command(cmd: Commands, set: &mut Vec<Task>, filpath: &mut PathBuf, auto_save: bool) -> Result<()> {
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
                if verbose {
                    println!("Verbose: Adding task {}: '{}'", i + 1, task_str);
                }
                
                let task = Task::from_string(task_str)?;
                set.push(task);
                added_count += 1;
            }
            
            if force {
                println!("Force: Adding {} tasks even if they exist", added_count);
            }
            
            if added_count == 1 {
                println!("You'd like to add {}", tasks[0]);
            } else {
                println!("Added {} tasks to the list", added_count);
            }
            
            if auto_save {
                let squash = set.iter().map(|t| t.to_file_string()).collect::<Vec<_>>().join("\n");
                fs::write(&filpath, squash)
                    .with_context(|| format!("Failed to write to {}", filpath.display()))?;
                if added_count == 1 {
                    println!("Task added and saved to {:?}", filpath.display());
                } else {
                    println!("Tasks added and saved to {:?}", filpath.display());
                }
            }
        }
        Commands::Print { verbose, sync, yes } => {
            if verbose {
                println!("Printing with verbose flag");
            }
            if sync {
                println!("Printing with sync flag");
            }
            if yes {
                println!("Printing with yes flag");
            }
            
            if set.is_empty() {
                println!("No tasks to print");
            } else {
                print_tasks(&set);
            }
        }
        Commands::Save { file } => {
            let target_path = if let Some(file_path) = file {
                PathBuf::from(&file_path)
            } else {
                filpath.clone()
            };
            
            if set.is_empty() {
                println!("Please enter at least 1 task before saving");
                return Ok(());
            }
            
            let squash = set.iter().map(|t| t.to_file_string()).collect::<Vec<_>>().join("\n");
            fs::write(&target_path, squash)
                .with_context(|| format!("Failed to write to {}", target_path.display()))?;
            println!("saving to {:?}", target_path.display());
        }
        Commands::Open { file } => {
            *filpath = PathBuf::from(&file);
            let file_handle = File::open(&filpath)
                .with_context(|| format!("Unable to open file: {}", file))?;
            let buf = BufReader::new(file_handle);
            println!("Opening file: {}", file);
            
            let lines: Vec<String> = buf
                .lines()
                .collect::<std::result::Result<Vec<_>, _>>()
                .with_context(|| "Failed to read file lines")?;
            
            *set = lines
                .into_iter()
                .filter(|line| !line.trim().is_empty())
                .map(|line| Task::from_string(&line))
                .collect::<Result<Vec<_>, _>>()?;
            
            print_tasks(&set);
        }
        Commands::Remove { task } => {
            let (removed_text, success) = remove_task(set, &task);
            
            if success {
                println!("Removed task: {}", removed_text);
                
                if auto_save {
                    let squash = set.iter().map(|t| t.to_file_string()).collect::<Vec<_>>().join("\n");
                    fs::write(&filpath, squash)
                        .with_context(|| format!("Failed to write to {}", filpath.display()))?;
                    println!("Task removed and saved to {:?}", filpath.display());
                }
            } else {
                println!("{}", removed_text);
            }
        }
        Commands::Clear => {
            set.clear();
            println!("Cleared task list");
            
            if auto_save {
                let squash = set.iter().map(|t| t.to_file_string()).collect::<Vec<_>>().join("\n");
                fs::write(&filpath, squash)
                    .with_context(|| format!("Failed to write to {}", filpath.display()))?;
                println!("Task list cleared and saved to {:?}", filpath.display());
            }
        }
        Commands::Which => {
            println!("Current file: {}", filpath.display());
        }
        Commands::Notify { minutes_before, verbose } => {
            // Create lock file to prevent overlapping cron jobs
            let lock_file = filpath.with_extension("lock");
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
            for task in set.iter() {
                if task.is_overdue() {
                    overdue_tasks.push(task);
                } else if task.is_due_soon(minutes_before) {
                    due_soon_tasks.push(task);
                }
            }
            
            // Send notifications for overdue tasks
            for task in overdue_tasks {
                if verbose {
                    println!("Sending overdue notification for: {}", task.text);
                }
                task.send_notification(true)?;
                notifications_sent += 1;
            }
            
            // Send notifications for tasks due soon
            for task in due_soon_tasks {
                if verbose {
                    println!("Sending due soon notification for: {}", task.text);
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
    }
    Ok(())
}
