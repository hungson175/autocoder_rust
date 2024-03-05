#[macro_export]
macro_rules! get_function_string {
    ($func: ident) => {
        stringify!($func)
    };
}

#[macro_use]
mod ai_functions;
mod apis;
mod helpers;
mod models;
use helpers::command_line::get_use_response;

use crate::models::agents_manager::managing_agent::ManagingAgent;
#[tokio::main]
async fn main() {
    // println!("Hello, world!");
    let user_response = get_use_response("What kind of website do you want to create?");
    let mut manager = ManagingAgent::new(user_response)
        .await
        .expect("Failed to create Managing Agent");
    
    manager.execute_project().await;

    // dbg!(manager);
}
