mod dispatcher;
mod parser;
mod terminal;

use terminal::Terminal;

use std::io::{self, Write};

fn main() {

    let mut terminal = Terminal::new();

    println!("Trimangees Terminal");

    loop {

        print!("> ");

        io::stdout().flush().unwrap();

        let mut input = String::new();

        io::stdin().read_line(&mut input).unwrap();

        terminal.run_command(
            input.trim(),
        );

    }

}