use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::{env, fs, fs::File, io::BufRead, io::BufReader, path::PathBuf};

/// Kani No Yotei - A simple todo list manager
#[derive(Parser)]
#[command(name = "kny")]
#[command(about = "A simple todo list manager")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,
}

#[derive(Subcommand)]
enum Commands {
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
        /// Task to remove
        task: String,
    },
    /// Clear all tasks from the list
    Clear,
    /// Show current working file
    Which,
}

/// Prints all tasks in the todo list
fn print(printable_set: &Vec<String>) {
    for m in printable_set {
        println!("{}", m);
    }
}

/// Prints help information for the program by reading from help.txt
fn print_help() {
    match fs::read_to_string("help.txt") {
        Ok(help_content) => {
            print!("{}", help_content);
        }
        Err(_) => {
            // Fallback help if help.txt is not found
            println!("Kani No Yotei - A simple todo list manager");
            println!();
            println!("USAGE:");
            println!("    kny [OPTIONS]");
            println!("    kny <COMMAND> [ARGS] [FLAGS]");
            println!();
            println!("OPTIONS:");
            println!("    --help, -h    Print this help message");
            println!();
            println!("COMMANDS:");
            println!("    add \"<task>\" [\"<task2>\"] [flags]  Add one or more tasks (quotes required)");
            println!("    print [flags]         Print all current tasks");
            println!("    save [file]           Save tasks to file (default: ToDo.txt)");
            println!("    open <file>           Open a different todo file");
            println!("    remove <task>         Remove a task from the list");
            println!("    clear                 Clear all tasks from the list");
            println!("    which                 Show current working file");
            println!("    exit, quit, e, q      Exit the program (REPL only)");
            println!();
            println!("For detailed help, ensure help.txt is in the current directory.");
        }
    }
}

/// Execute a clap command with proper error handling
fn execute_clap_command(cmd: Commands, set: &mut Vec<String>, filpath: &mut PathBuf, auto_save: bool) -> Result<()> {
    match cmd {
        Commands::Add { tasks, verbose, force } => {
            if tasks.is_empty() {
                println!("Error: No tasks provided");
                println!("Usage: kny add \"<task>\" [\"<task2>\"] [flags]");
                return Ok(());
            }
            
            let mut added_count = 0;
            for (i, task) in tasks.iter().enumerate() {
                if verbose {
                    println!("Verbose: Adding task {}: '{}'", i + 1, task);
                }
                set.push(task.clone());
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
                let squash = set.join("\n");
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
                print(&set);
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
            
            let squash = set.join("\n");
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
            *set = buf
                .lines()
                .collect::<std::result::Result<Vec<_>, _>>()
                .with_context(|| "Failed to read file lines")?;
            print(&set);
        }
        Commands::Remove { task } => {
            set.retain(|t| t != &task);
            println!("Removed task: {}", task);
            
            if auto_save {
                let squash = set.join("\n");
                fs::write(&filpath, squash)
                    .with_context(|| format!("Failed to write to {}", filpath.display()))?;
                println!("Task removed and saved to {:?}", filpath.display());
            }
        }
        Commands::Clear => {
            set.clear();
            println!("Cleared task list");
            
            if auto_save {
                let squash = set.join("\n");
                fs::write(&filpath, squash)
                    .with_context(|| format!("Failed to write to {}", filpath.display()))?;
                println!("Task list cleared and saved to {:?}", filpath.display());
            }
        }
        Commands::Which => {
            println!("Current file: {}", filpath.display());
        }
    }
    Ok(())
}

/// Simple REPL command parser for interactive mode
fn parse_repl_command(line: &str) -> Option<Commands> {
    let parts: Vec<&str> = line.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }
    
    match parts[0] {
        "add" => {
            let mut tasks = Vec::new();
            let mut verbose = false;
            let mut force = false;
            
            for part in &parts[1..] {
                if part.starts_with('-') {
                    if part.contains('v') { verbose = true; }
                    if part.contains('f') { force = true; }
                } else {
                    tasks.push(part.to_string());
                }
            }
            
            Some(Commands::Add { tasks, verbose, force })
        }
        "print" => {
            let mut verbose = false;
            let mut sync = false;
            let mut yes = false;
            
            for part in &parts[1..] {
                if part.starts_with('-') {
                    if part.contains('v') { verbose = true; }
                    if part.contains('S') { sync = true; }
                    if part.contains('y') { yes = true; }
                }
            }
            
            Some(Commands::Print { verbose, sync, yes })
        }
        "save" => {
            let file = if parts.len() > 1 { Some(parts[1].to_string()) } else { None };
            Some(Commands::Save { file })
        }
        "open" => {
            if parts.len() > 1 {
                Some(Commands::Open { file: parts[1].to_string() })
            } else {
                None
            }
        }
        "remove" => {
            if parts.len() > 1 {
                Some(Commands::Remove { task: parts[1].to_string() })
            } else {
                None
            }
        }
        "clear" => Some(Commands::Clear),
        "which" => Some(Commands::Which),
        _ => None,
    }
}

fn main() -> Result<()> {
    // Parse command line arguments with clap
    let cli = Cli::parse();
    
    // Handle command line arguments
    if let Some(command) = cli.command {
        // One-off command mode
        let mut set = Vec::<String>::new();
        let cur_dir = env::current_dir().unwrap();
        let mut filpath = cur_dir.join("ToDo.txt");
        
        // Load existing tasks if file exists
        if filpath.exists() {
            let file = File::open(&filpath)
                .with_context(|| "Unable to open ToDo.txt")?;
            let buf = BufReader::new(file);
            set = buf
                .lines()
                .collect::<std::result::Result<Vec<_>, _>>()
                .with_context(|| "Failed to read file lines")?;
        }
        
        // Execute command with auto-save enabled for one-off mode
        execute_clap_command(command, &mut set, &mut filpath, true)?;
        return Ok(());
    }
    
    // REPL mode
    let mut history = Editor::<(), rustyline::history::DefaultHistory>::new()?;
    if history.load_history("history.txt").is_err() {
        println!("No history found");
    }
    
    // Initialize task storage and file path
    let mut set = Vec::<String>::new();
    let cur_dir = env::current_dir().unwrap();
    let mut filpath = cur_dir.join("ToDo.txt");
    
    // Automatically open ToDo.txt on startup
    if filpath.exists() {
        let file = File::open(&filpath)
            .with_context(|| "Unable to open ToDo.txt")?;
        let buf = BufReader::new(file);
        println!("Opening default file: ToDo.txt");
        set = buf
            .lines()
            .collect::<std::result::Result<Vec<_>, _>>()
            .with_context(|| "Failed to read file lines")?;
        if !set.is_empty() {
            print(&set);
        }
    } else {
        println!("ToDo.txt not found, starting with empty task list");
    }
    
    // Main command loop - continues until user exits
    loop {
        let readline = history.readline(">> ");
        match readline {
            Ok(line) => {
                let _ = history.add_history_entry(&line);
                
                // Handle exit commands separately
                match line.as_str() {
                    "e" | "exit" | "q" | "quit" => std::process::exit(1),
                    _ => {
                        // Try to parse as a REPL command
                        if let Some(command) = parse_repl_command(&line) {
                            if let Err(e) = execute_clap_command(command, &mut set, &mut filpath, false) {
                                eprintln!("Error: {}", e);
                            }
                        } else {
                            println!("Unknown command: {}", line);
                            println!("Use --help for usage information");
                        }
                    }
                }
            }
            // Handle Ctrl+C interruption
            Err(ReadlineError::Interrupted) => {
                println!("C+C");
                break;
            }
            // Handle Ctrl+D (EOF)
            Err(ReadlineError::Eof) => {
                println!("C+D");
                break;
            }
            // Handle other readline errors
            Err(err) => {
                eprintln!("error: {:?}", err);
                break;
            }
        }
    }
    
    // Save command history before exiting
    history.save_history("history.txt")?;
    Ok(())
}
