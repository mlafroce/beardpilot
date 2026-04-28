use schemars::{JsonSchema, Schema};
use serde_json::Value;
use std::{future::Future, pin::Pin};

pub trait ErasedTool: Send {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn params_schema(&self) -> Schema;

    fn call_erased(
        &mut self,
        parameters: Value, // JSON crudo en lugar de Params tipado
    ) -> Pin<
        Box<
            dyn Future<Output = Result<String, Box<dyn std::error::Error + Send + Sync>>>
                + Send
                + '_,
        >,
    >;
}

impl<T> ErasedTool for T
where
    T: Tool + Send,
    T::Params: for<'de> serde::Deserialize<'de>,
{
    fn name(&self) -> &'static str {
        Tool::name(self)
    }

    fn description(&self) -> &'static str {
        Tool::description(self)
    }

    fn params_schema(&self) -> Schema {
        Tool::params_schema(self)
    }

    fn call_erased(
        &mut self,
        parameters: Value,
    ) -> Pin<
        Box<
            dyn Future<Output = Result<String, Box<dyn std::error::Error + Send + Sync>>>
                + Send
                + '_,
        >,
    > {
        Box::pin(async move {
            // Type erasure: deserializamos el JSON al tipo concreto
            let typed: T::Params = serde_json::from_value(parameters)
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)?;

            self.call(typed)
                .await
                .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        })
    }
}

pub trait Tool {
    //type Params: for<'de> Deserialize<'de> + JsonSchema;
    type Params: JsonSchema;
    type Error: std::error::Error + Send + Sync + 'static;

    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn params_schema(&self) -> Schema {
        schemars::schema_for!(Self::Params)
    }

    fn call(
        &mut self,
        parameters: Self::Params,
    ) -> impl Future<Output = Result<String, Self::Error>> + Send;
}

pub fn tool_to_json(tool: &dyn ErasedTool) -> serde_json::Value {
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
