#![allow(dead_code)]

use ballcube::Board;

fn build_shell() -> Option<Board> {
    let mut rl = rustyline::Editor::<()>::new();

    loop {
        let readline = rl.readline("build > ");
        // let readline = rl.readline("\u{1F6E0}> ");
        match readline {
            Ok(line) => {
                println!("Line: {}", line);
            }
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
    None
}

fn cli() {
    let mut rl = rustyline::Editor::<()>::new();
    loop {
        let readline = rl.readline("> ");
        match readline {
            Ok(line) => match line.as_str() {
                "build" => {
                    build_shell();
                }
                _ => {
                    println!("Unknown command: {}", line)
                }
            },
            Err(err) => {
                println!("Error: {:?}", err);
                break;
            }
        }
    }
}

fn main() {
    solver::machine_learning::generate_case_list();
}
