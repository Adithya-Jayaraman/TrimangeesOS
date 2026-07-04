use crate::dispatcher::CommandDispatcher;
use crate::parser::parse;

pub struct Terminal {
    dispatcher: CommandDispatcher,
}

impl Terminal {

    pub fn new() -> Self {

        Self {

            dispatcher: CommandDispatcher::new(),

        }

    }

    pub fn run_command(
        &mut self,
        input: &str,
    ) {

        let parsed = parse(input);

        self.dispatcher.dispatch(parsed);

    }

}