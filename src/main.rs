use anyhow::Result;
use clap::Parser;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::env;

use kani_no_yotei::{Cli, TaskStore, execute_clap_command, parse_repl_command};

fn main() -> Result<()> {
    // Parse command line arguments with clap
    let cli = Cli::parse();
    
    // Handle command line arguments
    if let Some(command) = cli.command {
        // One-off command mode
        let cur_dir = env::current_dir().unwrap();
        let mut filepath = cur_dir.join("tasks.json");
        
        // Load existing tasks if file exists, otherwise create new store
        let mut store = TaskStore::load_from_file(&filepath)?;
        
        // Execute command with auto-save enabled for one-off mode
        execute_clap_command(command, &mut store, &mut filepath, true, cli.debug)?;
        return Ok(());
    }
    
    // REPL mode
    let mut history = Editor::<(), rustyline::history::DefaultHistory>::new()?;
    if history.load_history("history.txt").is_err() {
        println!("No history found");
    }
    
    // Initialize task storage and file path
    let cur_dir = env::current_dir().unwrap();
    let mut filepath = cur_dir.join("tasks.json");
    
    // Load existing tasks if file exists, otherwise create new store
    let mut store = TaskStore::load_from_file(&filepath)?;
    
    if store.is_empty() {
        println!("No tasks found, starting with empty task list");
    } else {
        println!("Loaded {} tasks from tasks.json", store.len());
    }
    
    // Main command loop - continues until user exits
    loop {
        let readline = history.readline(">> ");
        match readline {
            Ok(line) => {
                let _ = history.add_history_entry(&line);
                
                // Handle exit commands separately
                match line.as_str() {
                    "e" | "exit" | "q" | "quit" => std::process::exit(0),
                    _ => {
                        // Try to parse as a REPL command
                        if let Some(command) = parse_repl_command(&line) {
                            if let Err(e) = execute_clap_command(command, &mut store, &mut filepath, false, false) {
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
