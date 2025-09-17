use crate::cli::Commands;

/// Parse a line into parts respecting quotes
pub fn parse_quoted_line(line: &str) -> Vec<String> {
    let mut parts = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut quote_char = '"';
    let mut chars = line.chars().peekable();
    
    while let Some(c) = chars.next() {
        match c {
            '"' | '\'' => {
                if !in_quotes {
                    // Starting quotes
                    in_quotes = true;
                    quote_char = c;
                } else if c == quote_char {
                    // Ending quotes
                    in_quotes = false;
                } else {
                    // Different quote type inside quotes - treat as literal
                    current.push(c);
                }
            }
            ' ' => {
                if in_quotes {
                    // Space inside quotes - treat as literal
                    current.push(c);
                } else {
                    // Space outside quotes - end current part
                    if !current.is_empty() {
                        parts.push(current.clone());
                        current.clear();
                    }
                }
            }
            _ => {
                current.push(c);
            }
        }
    }
    
    // Add the last part if it exists
    if !current.is_empty() {
        parts.push(current);
    }
    
    parts
}

/// Simple REPL command parser for interactive mode
pub fn parse_repl_command(line: &str) -> Option<Commands> {
    let parts = parse_quoted_line(line);
    if parts.is_empty() {
        return None;
    }
    
    match parts[0].as_str() {
        "add" => {
            let mut tasks = Vec::new();
            let mut verbose = false;
            let mut force = false;
            
            for part in &parts[1..] {
                if part.starts_with('-') {
                    if part.contains('v') { verbose = true; }
                    if part.contains('f') { force = true; }
                } else {
                    tasks.push(part.clone());
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
        "notify" => {
            let mut minutes_before = 15;
            let mut verbose = false;
            
            for part in &parts[1..] {
                if part.starts_with('-') {
                    if part.contains('v') { verbose = true; }
                    if part.contains('m') {
                        // Look for -m followed by number
                        if let Some(next_part) = parts.get(parts.iter().position(|p| p == part).unwrap() + 1) {
                            if let Ok(num) = next_part.parse::<u32>() {
                                minutes_before = num;
                            }
                        }
                    }
                }
            }
            
            Some(Commands::Notify { minutes_before, verbose })
        }
        _ => None,
    }
}
