use reqwest::Client;
use serde::de::DeserializeOwned;
use serde::Deserialize;

use crate::apis::call_request::call_gpt;
use crate::helpers::command_line::PrintCommand;
use crate::models::general::llm::Message;

const CODE_TEMPLATE_PATH: &str = "../web_template/src/code_template.rs";
// const EXEC_MAIN_PATH: &str = "/home/hungson175/dev/rust_autogpt/web_template/src/main.rs";
const EXEC_MAIN_PATH: &str = "../web_template/src/main.rs";
// pub const WEB_SERVER_PROJECT_PATH: &str = "/home/hungson175/dev/rust_autogpt/web_template/";
pub const WEB_SERVER_PROJECT_PATH: &str = "../web_template/";
// const API_SCHEMA_PATH: &str = "/home/hungson175/dev/rust_autogpt/auto_gippity/schemas/api_schema.json";
const API_SCHEMA_PATH: &str = "../web_template/api_schema.json";

// Extend ai function to encourage specific output
pub fn extend_ai_function(ai_func: fn(&str) -> &'static str, func_input: &str) -> Message {
    let ai_function_str = ai_func(func_input);
    // dbg!(ai_function_str);

    // Extend the string to encourage only printing the output
    let msg: String = format!(
        "FUNCTION: {}\n
        INPUT: {}\n
    INSTRUCTION: You are a function printer. You ONLY print the results of the function with the given input.
    Nothing else. No commentary.
    Print out what the function will return.",
        ai_function_str, func_input
    );

    Message {
        role: "system".to_string(),
        content: msg,
    }
}

pub async fn ai_task_request(
    msg_context: String,
    agent_position: &str,
    agent_operation: &str,
    function_pass: for<'a> fn(&'a str) -> &'static str,
) -> String {
    let extended_msg: Message = extend_ai_function(function_pass, &msg_context);

    PrintCommand::AICall.print_agent_message(agent_position, agent_operation);

    let llm_response_res: Result<String, Box<dyn std::error::Error + Send>> =
        call_gpt(vec![extended_msg.clone()]).await;

    match llm_response_res {
        Ok(llm_res_str) => llm_res_str,
        Err(_) => call_gpt(vec![extended_msg.clone()])
            .await
            .expect("Failed 2 to call OpenAI"),
    }
}

pub async fn ai_task_request_decoded<T: DeserializeOwned>(
    msg_context: String,
    agent_position: &str,
    agent_operation: &str,
    function_pass: for<'a> fn(&'a str) -> &'static str,
) -> T {
    let llm_response: String =
        ai_task_request(msg_context, agent_position, agent_operation, function_pass).await;
    let decoded_response: T =
        serde_json::from_str(llm_response.as_str()).expect("Failed to decode LLM response");
    return decoded_response;
}

// Check whether request url is valid
pub async fn check_status_code(client: &Client, url: &str) -> Result<u16, reqwest::Error> {
    let response = client.get(url).send().await?;
    Ok(response.status().as_u16())
}

// Get Code Template
pub fn read_code_template_content() -> String {
    std::fs::read_to_string(CODE_TEMPLATE_PATH).expect("Failed to read code template at ")
}

// Get exec main code
pub fn read_exc_main_content() -> String {
    std::fs::read_to_string(EXEC_MAIN_PATH).expect("Failed to read main code")
}

// Save New Backend Code
pub fn save_backend_code(content: &str) {
    std::fs::write(EXEC_MAIN_PATH, content).expect("Failed to write main.rs file")
}

// Save JSON API Endpoint Schema
pub fn save_api_endpoints(api_endpoints: &str) {
    std::fs::write(API_SCHEMA_PATH, api_endpoints).expect("Failed to write api_schema.json file")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ai_functions::aifunc_architect::print_project_scope;
    use crate::ai_functions::aifunc_managing::convert_user_input_to_goal;
use std::fs;

    #[test]
    fn test_extending_ai_function() {
        let extended_msg = extend_ai_function(convert_user_input_to_goal, "dummy variable");
        dbg!(&extended_msg);
        assert_eq!(extended_msg.role, "system".to_string());
    }

    #[tokio::test]
    async fn test_ai_task_request() {
        // Arrange
        let msg_context = "Build me a webserver for making Bitcoin price api requests".to_string();
        let agent_position = "Managing Agent";
        let agent_operation = "Defining user requirements";
        let function_pass = convert_user_input_to_goal;

        // Act
        // let response = ai_task_request(msg_context.to_string(), agent_position, agent_operation, function_pass).await;
        // dbg!(&response);

        let arch_response = ai_task_request(
            "build a website that makes Bitcoin price API requests".to_string(),
            "Architect Agent",
            "Build architect requirements",
            print_project_scope,
        )
        .await;
        dbg!(&arch_response);
        assert!(true);
    }

    #[test]
    fn test_save_api_endpoints() {
        // Arrange
        let api_endpoints = r#"{
            "endpoint1": "http://example.com/endpoint1",
            "endpoint2": "http://example.com/endpoint2"
        }"#;

        // Act
        save_api_endpoints(api_endpoints);

        // Assert
        let saved_endpoints = fs::read_to_string(API_SCHEMA_PATH).expect("Failed to read api_schema.json file");
        assert_eq!(saved_endpoints, api_endpoints);
    }

}

