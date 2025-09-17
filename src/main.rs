use anyhow::{Context, Result};
use clap::Parser;
use rustyline::error::ReadlineError;
use rustyline::Editor;
use std::{env, fs::File, io::BufRead, io::BufReader};

use kani_no_yotei::{Cli, Task, execute_clap_command, parse_repl_command};

fn main() -> Result<()> {
    // Parse command line arguments with clap
    let cli = Cli::parse();
    
    // Handle command line arguments
    if let Some(command) = cli.command {
        // One-off command mode
        let mut set = Vec::<Task>::new();
        let cur_dir = env::current_dir().unwrap();
        let mut filpath = cur_dir.join("ToDo.txt");
        
        // Load existing tasks if file exists
        if filpath.exists() {
            let file = File::open(&filpath)
                .with_context(|| "Unable to open ToDo.txt")?;
            let buf = BufReader::new(file);
            let lines: Vec<String> = buf
                .lines()
                .collect::<std::result::Result<Vec<_>, _>>()
                .with_context(|| "Failed to read file lines")?;
            
            set = lines
                .into_iter()
                .filter(|line| !line.trim().is_empty())
                .map(|line| Task::from_string(&line))
                .collect::<Result<Vec<_>, _>>()?;
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
    let mut set = Vec::<Task>::new();
    let cur_dir = env::current_dir().unwrap();
    let mut filpath = cur_dir.join("ToDo.txt");
    
    // Automatically open ToDo.txt on startup
    if filpath.exists() {
        let file = File::open(&filpath)
            .with_context(|| "Unable to open ToDo.txt")?;
        let buf = BufReader::new(file);
        println!("Opening default file: ToDo.txt");
        let lines: Vec<String> = buf
            .lines()
            .collect::<std::result::Result<Vec<_>, _>>()
            .with_context(|| "Failed to read file lines")?;
        
        set = lines
            .into_iter()
            .filter(|line| !line.trim().is_empty())
            .map(|line| Task::from_string(&line))
            .collect::<Result<Vec<_>, _>>()?;
        
        if !set.is_empty() {
            kani_no_yotei::task::print_tasks(&set);
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
                    "e" | "exit" | "q" | "quit" => std::process::exit(0),
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
