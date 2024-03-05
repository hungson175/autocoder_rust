use crossterm::{
    style::{Color, Print, ResetColor, SetForegroundColor},
    ExecutableCommand,
};

use std::io::{stdin, stdout};

#[derive(Debug, PartialEq)]
pub enum PrintCommand {
    AICall,
    UnitTest,
    Issue,
}

impl PrintCommand {
    pub fn print_agent_message(&self, agent_pos: &str, agent_statement: &str) {
        let mut stdout: std::io::Stdout = stdout();

        let statement_color = match self {
            PrintCommand::AICall => Color::Cyan,
            PrintCommand::UnitTest => Color::Magenta,
            PrintCommand::Issue => Color::Red,
        };

        stdout.execute(SetForegroundColor(Color::Green)).unwrap();
        print!("Agent: {}: ", agent_pos);
        stdout.execute(SetForegroundColor(statement_color)).unwrap();
        println!("{}", agent_statement);
        stdout.execute(ResetColor).unwrap();
    }
}

//Get user request
pub fn get_use_response(question: &str) -> String {
    let mut stdout: std::io::Stdout = std::io::stdout();

    //Print the question in a specific color
    stdout.execute(SetForegroundColor(Color::Cyan)).unwrap();
    println!("");
    println!("{}", question);

    //Reset the color
    stdout.execute(ResetColor).unwrap();

    //Read user input
    let mut user_response = String::new();
    stdin()
        .read_line(&mut user_response)
        .expect("Failed to read response");
    return user_response.trim().to_string();
}

// Get user review the AI-generated codes
pub fn confirm_safe_code() -> bool {
    let mut stdout: std::io::Stdout = std::io::stdout();

    loop {
        stdout.execute(SetForegroundColor(Color::Blue)).unwrap();
        println!("");
        println!("WARNING: you are about to execute code written ENTIRELY by AI.\n Please review the code carefully before executing it.");

        stdout.execute(ResetColor).unwrap();
        stdout.execute(SetForegroundColor(Color::Green)).unwrap();
        println!("[1] Ok, let's execute it.");
        stdout.execute(SetForegroundColor(Color::DarkRed)).unwrap();
        println!("[2] Stop the project !");

        //Read user input
        let mut user_response = String::new();
        stdin()
            .read_line(&mut user_response)
            .expect("Failed to read response");
        if user_response.trim().to_lowercase() == "1" {
            return true;
        } else if user_response.trim().to_lowercase() == "2" {
            return false;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_print_agent_message() {
        PrintCommand::AICall.print_agent_message("Managing Agent", "I am managing position");
    }

    #[test]
    fn test_get_use_response() {}
}
