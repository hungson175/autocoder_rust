use crate::ai_functions::aifunc_architect::{print_project_scope, print_site_urls};
use crate::helpers::command_line::PrintCommand;
use crate::helpers::general::{ai_task_request_decoded, check_status_code};
use crate::models::agent_basic::basic_agent::{AgentState, BasicAgent};
use crate::models::agent_basic::basic_trait::BasicTraits;
use crate::models::agents::agent_traits::{FactSheet, ProjectScope, SpecialFunctions};

use async_trait::async_trait;
use reqwest::Client;
use std::os::linux::raw::stat;
use std::time::Duration;

// Solution Architect
#[derive(Debug)]
pub struct AgentSolutionArchitect {
    attributes: BasicAgent,
}

impl AgentSolutionArchitect {
    pub fn new() -> Self {
        let attributes = BasicAgent {
            objective: "Gather information and design solutions for website development"
                .to_string(),
            position: "Solutions Architect".to_string(),
            state: AgentState::Discovery,
            memory: Vec::new(),
        };
        Self { attributes }
    }

    async fn call_project_scope(&mut self, factsheet: &mut FactSheet) -> ProjectScope {
        let msg_context = format!("{}", factsheet.project_description);
        let ai_response: ProjectScope = ai_task_request_decoded(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_project_scope),
            print_project_scope,
        )
        .await;

        factsheet.project_scope = Some(ai_response.clone());
        self.attributes.state = AgentState::Finished;

        return ai_response;
    }

    async fn call_determine_external_urls(
        &mut self,
        factsheet: &mut FactSheet,
        msg_context: String,
    ) {
        let ai_response: Vec<String> = ai_task_request_decoded(
            msg_context,
            &self.attributes.position,
            get_function_string!(print_site_urls),
            print_site_urls,
        )
        .await;

        factsheet.external_urls = Some(ai_response);
        self.attributes.state = AgentState::UnitTesting;
    }
}

#[async_trait]
impl SpecialFunctions for AgentSolutionArchitect {
    fn get_attributes_from_agent(&self) -> &BasicAgent {
        &self.attributes
    }

    async fn execute(
        &mut self,
        factsheet: &mut FactSheet,
    ) -> Result<(), Box<dyn std::error::Error>> {
        // !! WARNING: infinite loop -> infinite cost !!!!
        while self.attributes.state != AgentState::Finished {
            match self.attributes.state {
                AgentState::Discovery => {
                    let project_scope = self.call_project_scope(factsheet).await;

                    if project_scope.is_external_urls_required {
                        let msg_context = format!("{}", factsheet.project_description);
                        self.call_determine_external_urls(factsheet, msg_context)
                            .await;
                        self.attributes.state = AgentState::UnitTesting;
                    }
                }

                // We want to check if the external urls are valid, or LLM just hallucinated them
                AgentState::UnitTesting => {
                    let mut exclude_urls: Vec<String> = Vec::new();

                    let client = Client::builder()
                        .timeout(Duration::from_secs(5))
                        .build()
                        .unwrap();

                    let urls = factsheet
                        .external_urls
                        .as_ref()
                        .expect("No external urls found on factsheet");

                    for url in urls {
                        let endpoint_str = format!("Testing url: {}", url);
                        PrintCommand::UnitTest.print_agent_message(
                            self.attributes.position.as_str(),
                            endpoint_str.as_str(),
                        );

                        match check_status_code(&client, url).await {
                            Ok(status_code) => {
                                if status_code != 200 {
                                    exclude_urls.push(url.clone());
                                }
                            }
                            Err(e) => {
                                println!("Error checking {}: {}", url, e.to_string());
                                exclude_urls.push(url.clone());
                            }
                        }
                    }

                    if exclude_urls.len() > 0 {
                        let mut new_urls: Vec<String> = factsheet
                            .external_urls
                            .as_ref()
                            .unwrap()
                            .iter()
                            .filter(|url| !exclude_urls.contains(&url))
                            .cloned()
                            .collect();
                        factsheet.external_urls = Some(new_urls);
                    }

                    self.attributes.state = AgentState::Finished;
                }

                // Default to finished
                _ => {
                    self.attributes.state = AgentState::Finished;
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_solution_architect() {
        let mut agent: AgentSolutionArchitect = AgentSolutionArchitect::new();

        let mut factsheet: FactSheet = FactSheet {
            // project_description: "build a full stack website with user login and logout that shows lastest Forex prices".to_string(),
            project_description: "build a full stack website with user login and logout that shows lastest US stock prices".to_string(),
            project_scope: None,
            external_urls: None,
            backend_code: None,
            api_endpoint_schema: None
        };

        agent
            .execute(&mut factsheet)
            .await
            .expect("Unable to execute Solution Architect");
        assert!(factsheet.project_scope != None);
        assert!(factsheet.external_urls.is_some());

        dbg!(factsheet);
    }
}
