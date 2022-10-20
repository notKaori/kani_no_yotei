use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};
use std::{env, fs, fs::File, io::BufRead, io::BufReader, path::Path};

// Struct Session;

fn main() -> Result<()> {
    let mut history = Editor::<()>::new()?;
    if history.load_history("history.txt").is_err() {
        println!("No history found");
    }
    let mut set = Vec::<String>::new();
    let mut rl = history;
    let cur_dir = env::current_dir().unwrap();
    let mut filpath = cur_dir.as_path();
    loop {
        let readline = rl.readline(">> ");
        match readline {
            Ok(line) => {
                let qpath = filpath;
                let entry = line.as_str();
                rl.add_history_entry(entry);
                let spaces = entry.matches(" ").count();
                if spaces == 0 {
                    match entry {
                        "save" => {
                            let default_path;
                            let path;
                            if set.first() == None {
                                println!("Please enter at least 1 task before saving");
                                path = qpath;
                            } else if filpath == cur_dir.as_path() {
                                default_path = format!("{}/ToDo.txt", qpath.display());
                                path = Path::new(&default_path);
                            } else {
                                path = qpath;
                            }
                            let squash = set.join("\n");
                            let filstr = path;
                            println!("saving {:?}", filstr);
                            fs::write(filstr, squash).expect("Unable to write file");
                        }
                        "print" => {
                            for m in &set {
                                println!("{}", m);
                            }
                        }
                        _ => println!("incorrect"),
                    }
                } else {
                    let mut command = entry.splitn(2, " ");
                    let exec = command.next().unwrap();
                    let operand = command.next().unwrap();
                    match exec {
                        "add" => {
                            let to_do = operand.to_string();
                            println!("You'd like to add {}", to_do);
                            set.push(to_do);
                        }
                        "open" => {
                            filpath = Path::new(operand);
                            let file = File::open(filpath).expect("Unable to open file");
                            let buf = BufReader::new(file);
                            println!("Opening file: ");
                            set = buf
                                .lines()
                                .map(|l| l.expect("Could not parse line"))
                                .collect();
                            for m in &set {
                                println!("{}", m);
                            }
                        }
                        "print" => {
                            for m in &set {
                                println!("{}", m);
                            }
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
