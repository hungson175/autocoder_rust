use crate::models::general::llm::{APIResponse, ChatCompletion, Message};
use dotenv::dotenv;
use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use std::env;

//Call Large Language Model (i.e. GPT-4)
pub async fn call_gpt(messages: Vec<Message>) -> Result<String, Box<dyn std::error::Error + Send>> {
    dotenv().ok();

    let api_key =
        env::var("OPEN_AI_KEY").expect("OPENAI_API_KEY not found in environment variables");
    let api_org =
        env::var("OPEN_AI_ORG_ID").expect("OPENAI_ORG not found in environment variables");

    let url = "https://api.openai.com/v1/chat/completions";

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key))
            .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?,
    );
    headers.insert(
        "OpenAI-Organization",
        HeaderValue::from_str(&api_org.as_str())
            .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?,
    );

    let client = Client::builder()
        .default_headers(headers)
        .build()
        .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?;

    let chat_completion = ChatCompletion {
        model: "gpt-4".to_string(),
        messages,
        temperature: 0.1,
    };

    //Troubleshooting
    // let res_raw = client
    //     .post(url)
    //     .json(&chat_completion)
    //     .send()
    //     .await
    //     .unwrap();
    // dbg!(res_raw.text().await.unwrap());

    let response: APIResponse = client
        .post(url)
        .json(&chat_completion)
        .send()
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?
        .json()
        .await
        .map_err(|e| -> Box<dyn std::error::Error + Send> { Box::new(e) })?;
    Ok(response.choices[0].message.content.clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_call_gpt() {
        // Arrange
        // TODO: Initialize messages vector with test data
        let message = Message {
            role: "user".to_string(),
            content: "Hi there, this is a test. Give me a REAL short response".to_string(),
        };

        let mut messages = vec![];
        messages.push(message);

        // Act
        let response = call_gpt(messages).await;
        match response {
            Ok(res_str) => {
                dbg!(res_str);
                assert!(true);
            }
            Err(e) => {
                dbg!(e);
                assert!(false);
            }
        }

        // Assert
        // TODO: Add assertions based on the expected behavior of call_gpt
    }
}
