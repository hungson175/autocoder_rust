use crate::ai_functions::aifunc_backend::{
    print_backend_webserver_code, print_fixed_code, print_improved_webserver_code,
    print_rest_api_endpoints,
};

use crate::helpers::general::{
    check_status_code, read_code_template_content, read_exc_main_content, save_api_endpoints,
    save_backend_code, WEB_SERVER_PROJECT_PATH,
};

use crate::helpers::command_line::{confirm_safe_code, PrintCommand};
use crate::helpers::general::ai_task_request;
use crate::models::agent_basic::basic_agent::{AgentState, BasicAgent};
use crate::models::agents::agent_traits::{FactSheet, RouteObject, SpecialFunctions};

use async_trait::async_trait;
use reqwest::Client;
use std::fs;
use std::os::linux::raw::stat;
use std::process::{Command, Stdio};
use std::time::Duration;
use tokio::time;

#[derive(Debug)]
pub struct AgentBackendDev {
    attributes: BasicAgent,
    bug_errors: Option<String>,
    bug_count: u8,
}

impl AgentBackendDev {
    pub fn new() -> Self {
        let attributes = BasicAgent {
            objective: "Develop backend for the website".to_string(),
            position: "Backend Developer".to_string(),
            state: AgentState::Discovery,
            memory: Vec::new(),
        };
        Self {
            attributes,
            bug_errors: None,
            bug_count: 0,
        }
    }

    async fn call_initial_backend_code(&mut self, factsheet: &mut FactSheet) {
        // First version: junior dev

        let code_template_str = read_code_template_content();
        let external_urls: String = factsheet
            .external_urls
            .as_ref()
            .unwrap()
            .iter()
            .map(|url| format!("\"{}\"", url))
            .collect::<Vec<String>>()
            .join(", ");
        let project_desc = format!(
            "CODE_TEMPLATE: {} \n PROJECT_DESCRIPTION: {} \n OPTIONAL_EXTERNAL_URLS: {}",
            code_template_str, factsheet.project_description, external_urls
        );
        
        let backend_code: String = ai_task_request(
            project_desc,
            &self.attributes.position,
            get_function_string!(print_backend_webserver_code),
            print_backend_webserver_code,
        )
        .await;

        save_backend_code(&backend_code);
        factsheet.backend_code = Some(backend_code);
    }

    async fn call_improved_backend_code(&mut self, factsheet: &mut FactSheet) {
        // Here comes the senior dev

        let msg_context = format!(
            "CODE_TEMPLATE: {:?} \n PROJECT_DESCRIPTION: {:?}\n
            THIS FUNCTION ONLY PRINTS THE FIXED CODE. NOTHING ELSE. NO COMMENTARY.",
            factsheet.backend_code, factsheet
        );
        let backend_code: String = ai_task_request(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_improved_webserver_code),
            print_improved_webserver_code,
        )
        .await;

        save_backend_code(&backend_code);
        factsheet.backend_code = Some(backend_code);
    }

    async fn call_fix_code_bugs(&mut self, factsheet: &mut FactSheet) {
        let msg_context = format!(
            "BROKEN_CODE: {:?} \n ERROR_BUGS: {:?}\n
            THIS FUNCTION ONLY PRINTS THE FIXED CODE. NOTHING ELSE. NO COMMENTARY.",
            factsheet.backend_code, self.bug_errors
        );
        let backend_code: String = ai_task_request(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_fixed_code),
            print_fixed_code,
        )
        .await;

        save_backend_code(&backend_code);
        factsheet.backend_code = Some(backend_code);
    }

    async fn call_extract_rest_api_schema(&mut self) -> String {
        let msg_context = format!("CODE_INPUT: {}", read_exc_main_content());
        let api_schema: String = ai_task_request(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_rest_api_endpoints),
            print_rest_api_endpoints,
        )
        .await;
        api_schema
    }
}

#[async_trait]
impl SpecialFunctions for AgentBackendDev {
    fn get_attributes_from_agent(&self) -> &BasicAgent {
        &self.attributes
    }

    // This function will allow agent to execute their logic
    async fn execute(
        &mut self,
        factsheet: &mut FactSheet,
    ) -> Result<(), Box<dyn std::error::Error>> {
        while self.attributes.state != AgentState::Finished {
            match self.attributes.state {
                AgentState::Discovery => {
                    self.call_initial_backend_code(factsheet).await;
                    self.attributes.state = AgentState::Working;
                    continue;
                }
                AgentState::Working => {
                    if self.bug_count == 0 {
                        self.call_improved_backend_code(factsheet).await;
                    } else {
                        self.call_fix_code_bugs(factsheet).await;
                    }
                    self.attributes.state = AgentState::UnitTesting;
                    continue;
                }
                AgentState::UnitTesting => {
                    // Safe guard
                    PrintCommand::UnitTest.print_agent_message(
                        &self.attributes.position.as_str(),
                        "Backend Unit Testing: Need user input ...",
                    );

                    let is_safe_code = confirm_safe_code();
                    if !is_safe_code {
                        panic!("Work on AI ethics !");
                    }

                    // Build and test code
                    PrintCommand::UnitTest.print_agent_message(
                        &self.attributes.position.as_str(),
                        "Backend Unit Testing: building the project ...",
                    );

                    let build_backend_server: std::process::Output = Command::new("cargo")
                        .arg("build")
                        .current_dir(WEB_SERVER_PROJECT_PATH)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .output()
                        .expect("Failed to build backend application");

                    if build_backend_server.status.success() {
                        self.bug_count = 0;
                        PrintCommand::UnitTest.print_agent_message(
                            &self.attributes.position.as_str(),
                            "Backend Unit Testing: Backend server is built successfully",
                        );
                    } else {
                        let error_arr: Vec<u8> = build_backend_server.stderr;
                        let error_str = String::from_utf8(error_arr).unwrap();

                        self.bug_count += 1;
                        self.bug_errors = Some(error_str);

                        // Too many bug: wow, stop, I am not that rich !
                        if self.bug_count > 2 {
                            PrintCommand::Issue.print_agent_message(
                                &self.attributes.position.as_str(),
                                "Backend Unit Testing: Exit, too many bugs - AI becomes too expensive !",
                            );
                            panic!("Error: too many bugs!");
                        }

                        self.attributes.state = AgentState::Working;
                        continue;
                    }

                    // Extract and Test Rest API Endpoints'
                    let api_endpoints_str = self.call_extract_rest_api_schema().await;

                    let api_endpoints: Vec<RouteObject> =
                        serde_json::from_str(api_endpoints_str.as_str())
                            .expect("Failed to decode API Endpoints");

                    // Define endpoints to check
                    let check_endpoints: Vec<RouteObject> = api_endpoints
                        .iter()
                        .filter(|&route| {
                            (route.method == "GET" || route.method == "get")
                                && route.is_route_dynamic == "false"
                        })
                        .cloned()
                        .collect();

                    // Store API Endpoints: why just get methods ?
                    factsheet.api_endpoint_schema = Some(check_endpoints.clone()); //why cloned ? just take it ?

                    // Run backend application
                    PrintCommand::UnitTest.print_agent_message(
                        &self.attributes.position.as_str(),
                        "Backend Unit Testing: running the project ...",
                    );

                    let mut run_backend_server: std::process::Child = Command::new("cargo")
                        .arg("run")
                        .current_dir(WEB_SERVER_PROJECT_PATH)
                        .stdout(Stdio::piped())
                        .stderr(Stdio::piped())
                        .spawn()
                        .expect("Failed to run backend application");

                    PrintCommand::UnitTest.print_agent_message(
                        &self.attributes.position.as_str(),
                        "Backend Unit Testing: launching the project in 5 seconds ...",
                    );

                    let sleep_dur: Duration = Duration::from_secs(5);
                    time::sleep(sleep_dur).await;

                    // Check status code
                    for endpoint in check_endpoints {
                        let testing_msg: String = format!("Testing endpoint: '{}'", endpoint.route);
                        PrintCommand::UnitTest.print_agent_message(
                            &self.attributes.position.as_str(),
                            testing_msg.as_str(),
                        );

                        let client = Client::builder()
                            .timeout(Duration::from_secs(5))
                            .build()
                            .unwrap();

                        // Test url
                        let url = format!("http://localhost:8080{}", endpoint.route);
                        match check_status_code(&client, &url).await {
                            Ok(status_code) => {
                                if status_code != 200 {
                                    let err_msg: String = format!(
                                        "WARNING: Failed to call endpoint '{}'",
                                        endpoint.route
                                    );
                                    PrintCommand::Issue.print_agent_message(
                                        &self.attributes.position.as_str(),
                                        err_msg.as_str(),
                                    );
                                }
                            }
                            Err(e) => {
                                // kill $(lsof -t -i:8080)
                                run_backend_server
                                    .kill()
                                    .expect("Failed to kill backend web server");
                                let err_msg: String = format!("Error, checking backend '{}'", e);
                                PrintCommand::Issue.print_agent_message(
                                    &self.attributes.position.as_str(),
                                    err_msg.as_str(),
                                );
                            }
                        }
                    }
                    save_api_endpoints(&api_endpoints_str);
                    PrintCommand::UnitTest.print_agent_message(
                        &self.attributes.position.as_str(),
                        "Unit Testing: Backend testing is completed !",
                    );
                    run_backend_server
                        .kill()
                        .expect("Kill backend web server on completion");

                    self.attributes.state = AgentState::Finished;
                    continue;
                }
                _ => {
                    // self.attributes.state = AgentState::Finished;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::agent_basic::basic_agent::AgentState;
    use crate::models::agents::agent_traits::FactSheet;
    use crate::models::agents::agent_traits::{ProjectScope, SpecialFunctions};
    use crate::models::general::llm::Message;

    #[tokio::test]
    async fn test_writing_backend_code() {
        let mut agent = AgentBackendDev::new();
        // let mut factsheet = FactSheet {
        //     project_description: "build a website that displays current cryptocurrency prices (BTC, ETH, SOL, etc.) and provides time based on internet timezone.".to_string(),
        //     project_scope: Some(
        //         ProjectScope {
        //             is_crud_required: false,
        //             is_user_login_and_logout: false,
        //             is_external_urls_required: true,
        //         },
        //     ),
        //     external_urls: Some(
        //         vec![
        //             "https://api.binance.com/api/v3/exchangeInfo".to_string(),
        //             "https://api.binance.com/api/v3/klines?symbol=BTCUSDT&interval=1d".to_string(),
        //             "https://api.kraken.com/0/public/Ticker?pair=BTCUSD".to_string(),
        //             "https://api.kraken.com/0/public/Ticker?pair=ETHUSD".to_string(),
        //             "https://api.kraken.com/0/public/Ticker?pair=SOLUSD".to_string(),
        //             "http://worldtimeapi.org/api/timezone".to_string(),
        //         ],
        //     ),
        //     backend_code: None,
        //     api_endpoint_schema: None,
        // };
        // dbg!(factsheet);
        
        // let factsheet_str: &str =r#"
        // {
        //   "project_description": "build a website that fetches and tracks fitness progress with timezone information",
        //   "project_scope": {
        //     "is_crud_required": true,
        //     "is_user_login_and_logout": true,
        //     "is_external_urls_required": true
        //   },
        //   "external_urls": [
        //     "http://worldtimeapi.org/api/timezone"
        //   ],
        //   "backend_code": null,
        //   "api_endpoint_schema": null
        // }"#;

        let factsheet_str: &str =r#"
        {
          "project_description": "build a website which returns current time",
          "project_scope": {
            "is_crud_required": false,
            "is_user_login_and_logout": false,
            "is_external_urls_required": false
          },
          "external_urls": [],
          "backend_code": null,
          "api_endpoint_schema": null
        }"#;
        let mut factsheet: FactSheet = serde_json::from_str(factsheet_str).unwrap();

        agent.attributes.state = AgentState::Discovery;
        agent
            .execute(&mut factsheet)
            .await
            .expect("Failed to execute agent Backend Dev");
    }
}
