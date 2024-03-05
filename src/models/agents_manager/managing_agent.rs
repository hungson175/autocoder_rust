use crate::models::agent_basic::basic_agent::{AgentState, BasicAgent};
use crate::models::agents::agent_backend::AgentBackendDev;
use crate::models::agents::agent_traits::{FactSheet, SpecialFunctions};

use crate::ai_functions::aifunc_managing::convert_user_input_to_goal;
use crate::helpers::general::ai_task_request;
use crate::models::agents::agent_architect::AgentSolutionArchitect;
use crate::models::general::llm::Message;

#[derive(Debug)]
pub struct ManagingAgent {
    attributes: BasicAgent,
    factsheet: FactSheet,
    agents: Vec<Box<dyn SpecialFunctions>>,
}

impl ManagingAgent {
    pub async fn new(user_request: String) -> Result<Self, Box<dyn std::error::Error>> {
        let position = "Project Manager".to_string();
        let attributes = BasicAgent {
            objective: "Manage the project to build excellent website for the user".to_string(),
            position: position.clone(),
            state: AgentState::Discovery,
            memory: Vec::new(),
        };
        let ai_response = ai_task_request(
            user_request,
            &position,
            get_function_string!(convert_user_input_to_goal),
            convert_user_input_to_goal,
        )
        .await;

        let factsheet = FactSheet {
            project_description: ai_response,
            project_scope: None,
            external_urls: None,
            backend_code: None,
            api_endpoint_schema: None,
        };

        let agents: Vec<Box<dyn SpecialFunctions>> = vec![];
        Ok(Self {
            attributes,
            factsheet,
            agents,
        })
    }

    pub fn add_agent(&mut self, agent: Box<dyn SpecialFunctions>) {
        self.agents.push(agent);
    }

    fn create_agents(&mut self) {
        self.add_agent(Box::new(AgentSolutionArchitect::new()));
        self.add_agent(Box::new(AgentBackendDev::new()));        

        // Later on: can add anything: Testers, DevOps ...
    }

    pub async fn execute_project(&mut self) {
        self.create_agents();

        // Question: so the order of agent in the vector is important ?
        // Because, i.e: Backend agent should be called after the architect agent
        for agent in self.agents.iter_mut() {
            let agent_res: Result<(), Box<dyn std::error::Error>> =
                agent.execute(&mut self.factsheet).await;
            // let agent_info = agent.get_attributes_from_agent();
            // dbg!(agent_info);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn test_managing_agent() {
        let user_request = "build a website show current crypto currencies prices (BTC, ETH, SOL ....), and provides time based on internet timezone.".to_string();
        let mut manager = ManagingAgent::new(user_request)
            .await
            .expect("Failed to create Managing Agent");
        manager.execute_project().await;
        dbg!(manager.factsheet);
    }
}
