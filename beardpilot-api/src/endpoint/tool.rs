use std::future::Future;
use schemars::{JsonSchema, Schema};

pub trait Tool: Send {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn params_schema(&self) -> Schema;
}

pub trait ParamTypedTool: Tool {
    //type Params: for<'de> Deserialize<'de> + JsonSchema;
    type Params: JsonSchema;
    type Error: std::error::Error + Send + Sync + 'static;

    fn call(&mut self, parameters: Self::Params) -> impl Future<Output = Result<String, Self::Error>> + Send;
}

pub fn tool_to_json(tool: &dyn Tool) -> serde_json::Value {
    let mut schema = tool.params_schema();
    let _ = schema.remove("$schema");
    let obj = schema.as_value();

    serde_json::json!({
        "type": "function",
        "function": {
            "name": tool.name(),
            "description": tool.description(),
            "parameters": obj
        }
    })
}
