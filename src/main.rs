use rustyline::error::ReadlineError;
use rustyline::{Editor, Result};
use std::{env, fs, fs::File, io::BufRead, io::BufReader, path::Path};

struct Session {
    set: Vec<String>,
    command: str,
}

fn print(printable_set: &Vec<String>) {
    for m in printable_set {
        println!("{}", m);
    }
}

fn parse_command(command: String) -> Result<()> {
    match command.as_str() {
        Ok(line) => {
            // history.add_history_entry(&line);
            // let qpath = &filpath;
            let entry = line.as_str();
            //history.add_history_entry(entry);
            let spaces = entry.matches(" ").count();
            if spaces == 0 {
                match entry {
                    // "save" => {
                    //     let path_str = format!("{}/ToDo.txt", qpath.display());
                    //     let default_path = Path::new(&path_str);
                    //     let path;
                    //     if set.first() == None {
                    //         println!("Please enter at least 1 task before saving");
                    //         path = qpath;
                    //     } else if filpath == cur_dir.as_path() {
                    //         path = &default_path;
                    //     } else {
                    //         path = qpath;
                    //     }
                    //     let squash = set.join("\n");
                    //     let filstr = path;
                    //     println!("saving {:?}", filstr);
                    //     fs::write(filstr, squash).expect("Unable to write file");
                    // }
                    "print" => {
                        print(&set);
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
                        print(&set);
                    }
                    "print" => print(&set),
                    "q" => std::process::exit(1),
                    "quit" => std::process::exit(1),
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
    }
}

fn main() -> Result<()> {
    let mut history = Editor::<()>::new()?;
    if history.load_history("history.txt").is_err() {
        println!("No history found");
    }
    let mut set = Vec::<String>::new();
    //let mut rl = history;
    let cur_dir = env::current_dir().unwrap();
    let mut filpath = cur_dir.as_path();
    loop {
        let readline = history.readline(">> ");
        match parse_command(readline.unwrap()) {
            Ok{}
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
    history.save_history("history.txt")
}
