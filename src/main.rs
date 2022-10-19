use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};
use std::{fs, fs::File, io::BufRead, io::BufReader, path::Path};

fn print(printable_set: &Vec<String>) -> Result<()> {
    if printable_set.len < 1 {
        println!("Cannot print empty list");
        return;
    }
    for m in printable_set {
        println!("{}", m)
    }
}

fn main() -> Result<()> {
    let mut history = Editor::<()>::new()?;
    if history.load_history("history.txt").is_err() {
        println!("No history found");
    }
    let mut set = Vec::<String>::new();
    let mut rl = history;
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let entry = line.as_str();
                rl.add_history_entry(entry);
                let mut command = line.splitn(2, " ");
                let exec = command.next().unwrap();
                let operand = command.next().unwrap();
                match exec {
                    "add" => {
                        let to_do = operand.to_string();
                        println!("You'd like to add {}", to_do);
                        set.push(to_do);
                    }
                    "open" => {
                        let buf = BufReader::new(
                            File::open(Path::new(operand)).expect("Unable to open file"),
                        );
                        println!("Opening file: ");
                        set = buf
                            .lines()
                            .map(|l| l.expect("Could not parse line"))
                            .collect();
                        print(&set);
                        // for m in &set {
                        //     println!("{}", m);
                        // }
                    }
                    "print" => {
                        print(&set);
                        // for m in &set {
                        //     println!("{}", m);
                        // }
                    }
                    "q" => std::process::exit(1),
                    "save" => {
                        let squash = set.join("\n");
                        let path = Path::new(operand);
                        println!("saving {}", squash);
                        fs::write(path, squash).expect("Unable to write file");
                    }
                    _ => println!("huh?"),
                }
            }
            Err(ReadlineError::Interrupted) => {
                println!("C+C");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("C+D");
                break;
            }
            Err(err) => {
                println!("error: {:?}", err);
                break;
            }
        }
    }
    rl.save_history("history.txt")
}
