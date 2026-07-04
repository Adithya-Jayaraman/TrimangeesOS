use crate::parser::ParsedCommand;

// This import will point to your Shell crate.
// We'll refine the Cargo.toml setup later.
use shell::api::ShellApi;

pub struct CommandDispatcher {
    api: ShellApi,
}

impl CommandDispatcher {

    pub fn new() -> Self {

        Self {
            api: ShellApi::new(),
        }

    }

    pub fn dispatch(
        &mut self,
        parsed: ParsedCommand,
    ) {

        match parsed.command.as_str() {

            "help" => {

                println!("Commands:");
                println!("open");
                println!("help");

            }

            "open" => {

                if parsed.arguments.is_empty() {

                    println!("Open what?");

                } else {

                    self.api.launch_app(
                        &parsed.arguments[0]
                    );

                }

            }

            _ => {

                println!("Unknown command.");

            }

        }

    }

}