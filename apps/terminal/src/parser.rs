pub struct ParsedCommand {
    pub command: String,
    pub arguments: Vec<String>,
}

pub fn parse(input: &str) -> ParsedCommand {

    let words: Vec<&str> = input.split_whitespace().collect();

    if words.is_empty() {

        return ParsedCommand {
            command: String::new(),
            arguments: Vec::new(),
        };

    }

    let command = words[0].to_string();

    let mut arguments = Vec::new();

    for word in &words[1..] {
        arguments.push(word.to_string());
    }

    ParsedCommand {
        command,
        arguments,
    }

}