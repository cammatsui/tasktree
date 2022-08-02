use tasktree::command::{ Command, GENERAL_USAGE };
use std::{ env, process };

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        println!("{}", GENERAL_USAGE);
        process::exit(1);
    }

    let command = Command::from_args(args[1..].to_vec());
    match command {
        Err(msg) => println!("{}", msg),
        Ok(cmd) => {
            match cmd.execute() {
                Err(msg) => println!("Error: {}", msg),
                Ok(response) => println!("{}", response),
            }
        }
    }
}
