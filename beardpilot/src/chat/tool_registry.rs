use std::collections::HashMap;

use beardpilot_api::endpoint::{
    chat::{ToolCallFunction, ToolCallMessage},
    tool::{tool_to_json, ErasedTool},
};
use serde_json::Value;

use crate::{
    error::{AppError, AppResult},
    tools::list_files::ListFiles,
};

pub struct ToolRegistry {
    tools: HashMap<String, Box<dyn ErasedTool>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        let mut tools: HashMap<String, Box<dyn ErasedTool>> = HashMap::new();
        tools.insert("ListFiles".to_owned(), Box::new(ListFiles {}));
        Self { tools }
    }

    pub fn get_chat_tools(&self) -> Vec<Value> {
        self.tools
            .values()
            .map(|t| tool_to_json(t.as_ref()))
            .collect()
    }

    pub async fn call_tool(&mut self, call: ToolCall) -> AppResult<String> {
        let tool = self
            .tools
            .get_mut(&call.function)
            .ok_or(AppError::ToolError(format!(
                "Invalid tool: {}",
                call.function
            )))?;
        tool.call_erased(call.arguments)
            .await
            .map_err(|e| AppError::ToolError(e.to_string()))
    }
}

#[derive(Clone, PartialEq)]
pub struct ToolCall {
    pub id: String,
    pub function: String,
    pub arguments: Value,
}

impl ToolCall {
    pub fn to_string(&self) -> String {
        let args = match self.arguments.as_object() {
            Some(obj) => obj
                .iter()
                .map(|(key, value)| format!("{}={}", key, Self::format_json_value(value)))
                .collect::<Vec<_>>()
                .join(", "),
            None => String::new(),
        };
        format!("{}({})", &self.function, args)
    }

    pub fn to_tool_call_message(&self) -> ToolCallMessage {
        let function = ToolCallFunction {
            arguments: self.arguments.to_string(),
            index: None,
            name: self.function.clone(),
        };
        let message = ToolCallMessage {
            function,
            id: self.id.clone(),
            index: 0,
        };
        message
    }

    fn format_json_value(value: &Value) -> String {
        match value {
            Value::String(s) => format!("{:?}", s), // adds quotes + escaping
            _ => value.to_string(),
        }
    }
}
