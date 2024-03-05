use crate::models::agent_basic::basic_agent::BasicAgent;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;

///   {
///     "route": "/item/{id}",
///     "is_route_dynamic": "true",
///     "method": "get"
///     "request_body": "None",
///     "response": {
///       "id": "number",
///       "name": "string",
///       "completed": "bool",
///     }
///   },
///   {
///     "route": "/item",
///     "is_route_dynamic": "false",
///     "method": "post",
///     "request_body": {
///       "id": "number",
///       "name": "string",
///       "completed": "bool",
///     },
///     "response": "None"
///   },

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct RouteObject {
    pub route: String,
    pub is_route_dynamic: String,
    pub method: String,
    pub request_body: serde_json::Value,
    pub response: serde_json::Value,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct ProjectScope {
    pub is_crud_required: bool,
    pub is_user_login_and_logout: bool,
    pub is_external_urls_required: bool,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub struct FactSheet {
    pub project_description: String,
    pub project_scope: Option<ProjectScope>,
    pub external_urls: Option<Vec<String>>,
    pub backend_code: Option<String>,
    pub api_endpoint_schema: Option<Vec<RouteObject>>,
}

#[async_trait]
pub trait SpecialFunctions: Debug {
    // Used so that manager can get attributes from Agents
    fn get_attributes_from_agent(&self) -> &BasicAgent;

    // This function will allow agent to execute their logic
    async fn execute(
        &mut self,
        factsheet: &mut FactSheet,
    ) -> Result<(), Box<dyn std::error::Error>>;
}
