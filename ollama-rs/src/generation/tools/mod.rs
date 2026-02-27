#[cfg_attr(docsrs, doc(cfg(feature = "tool-implementations")))]
#[cfg(feature = "tool-implementations")]
pub mod implementations;

use std::{future::Future, pin::Pin};

use schemars::{generate::SchemaSettings, JsonSchema, Schema};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use serde_json::Value;

pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error + Send + Sync>>;

/// It's highly recommended that the `JsonSchema` has descriptions for all attributes.
/// Descriptions can be defined with `#[schemars(description = "Hi I am an attribute")]` above each attribute
// TODO enforce at compile-time
pub trait Tool: Send + Sync {
    type Params: Parameters;

    fn name() -> &'static str;
    fn description() -> &'static str;

    /// Call the tool.
    /// Note that returning an Err will cause it to be bubbled up. If you want the LLM to handle the error,
    /// return that error as a string.
    fn call(
        &mut self,
        parameters: Self::Params,
    ) -> impl Future<Output = Result<String>> + Send + Sync;
}

pub trait Parameters: DeserializeOwned + JsonSchema {}

impl<P: DeserializeOwned + JsonSchema> Parameters for P {}

pub(crate) trait ToolHolder: Send + Sync {
    fn call(
        &mut self,
        parameters: Value,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + '_ + Send + Sync>>;
}

impl<T: Tool> ToolHolder for T {
    fn call(
        &mut self,
        parameters: Value,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + '_ + Send + Sync>> {
        Box::pin(async move {
            // Json returned from the model can sometimes be in different formats, see https://github.com/pepperoni21/ollama-rs/issues/210
            // This is a work-around for this issue.
            let param_value = match serde_json::from_value(parameters.clone()) {
                // We first try with the ToolCallFunction format
                Ok(ToolCallFunction { name: _, arguments }) => arguments,
                Err(_err) => match serde_json::from_value::<ToolInfo>(parameters.clone()) {
                    Ok(ti) => ti.function.parameters.to_value(),
                    Err(_err) => parameters,
                },
            };

            let param = serde_json::from_value(param_value)?;

            T::call(self, param).await
        })
    }
}

/// A dynamic tool holder that wraps a closure for runtime-discovered tools.
///
/// This enables registering tools whose names and descriptions are not known at compile time,
/// such as MCP (Model Context Protocol) tools discovered from external servers.
pub struct DynamicToolHolder<F, Fut>
where
    F: FnMut(Value) -> Fut + Send + Sync,
    Fut: Future<Output = Result<String>> + Send + Sync,
{
    handler: F,
}

impl<F, Fut> DynamicToolHolder<F, Fut>
where
    F: FnMut(Value) -> Fut + Send + Sync,
    Fut: Future<Output = Result<String>> + Send + Sync,
{
    /// Create a new dynamic tool holder with the given handler closure.
    pub fn new(handler: F) -> Self {
        Self { handler }
    }
}

impl<F, Fut> ToolHolder for DynamicToolHolder<F, Fut>
where
    F: FnMut(Value) -> Fut + Send + Sync,
    Fut: Future<Output = Result<String>> + Send + Sync,
{
    fn call(
        &mut self,
        parameters: Value,
    ) -> Pin<Box<dyn Future<Output = Result<String>> + '_ + Send + Sync>> {
        Box::pin((self.handler)(parameters))
    }
}

impl ToolFunctionInfo {
    /// Create a ToolFunctionInfo from dynamic parameters.
    ///
    /// This is useful for runtime-discovered tools (like MCP tools) where
    /// the name and description are not known at compile time.
    pub fn from_dynamic(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: Schema,
    ) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            parameters,
        }
    }
}

impl ToolInfo {
    /// Create a ToolInfo from dynamic parameters.
    ///
    /// This is useful for runtime-discovered tools (like MCP tools) where
    /// the name and description are not known at compile time.
    pub fn from_dynamic(
        name: impl Into<String>,
        description: impl Into<String>,
        parameters: Schema,
    ) -> Self {
        Self {
            tool_type: ToolType::Function,
            function: ToolFunctionInfo::from_dynamic(name, description, parameters),
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolInfo {
    #[serde(rename = "type")]
    pub tool_type: ToolType,
    pub function: ToolFunctionInfo,
}

impl ToolInfo {
    pub(crate) fn new<P: Parameters, T: Tool<Params = P>>() -> Self {
        let mut settings = SchemaSettings::draft07();
        settings.inline_subschemas = true;
        let generator = settings.into_generator();

        let parameters = generator.into_root_schema_for::<P>();

        Self {
            tool_type: ToolType::Function,
            function: ToolFunctionInfo {
                name: T::name().to_string(),
                description: T::description().to_string(),
                parameters,
            },
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ToolType {
    Function,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolFunctionInfo {
    pub name: String,
    pub description: String,
    pub parameters: Schema,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCall {
    pub function: ToolCallFunction,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ToolCallFunction {
    pub name: String,
    // I don't love this (the Value)
    // But fixing it would be a big effort
    // FIXME
    #[serde(alias = "parameters")]
    pub arguments: Value,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_type_serializes_to_lowercase() {
        let tool_type = ToolType::Function;
        let json = serde_json::to_string(&tool_type).unwrap();
        assert_eq!(json, "\"function\"");
    }

    #[test]
    fn test_tool_type_deserializes_from_lowercase() {
        let tool_type: ToolType = serde_json::from_str("\"function\"").unwrap();
        assert!(matches!(tool_type, ToolType::Function));
    }

    #[test]
    fn test_tool_info_serializes_type_correctly() {
        // Use serde_json to create a minimal schema
        let schema_value = serde_json::json!({"type": "object"});
        let parameters: Schema = serde_json::from_value(schema_value).unwrap();

        let tool_info = ToolInfo {
            tool_type: ToolType::Function,
            function: ToolFunctionInfo {
                name: "test_tool".to_string(),
                description: "A test tool".to_string(),
                parameters,
            },
        };
        let json = serde_json::to_string(&tool_info).unwrap();
        assert!(json.contains("\"type\":\"function\""));
    }

    #[test]
    fn test_tool_info_matches_ollama_api_format() {
        // Verify the JSON structure matches Ollama API:
        // { "type": "function", "function": { "name": "...", "description": "...", "parameters": {...} } }
        let schema_value = serde_json::json!({
            "type": "object",
            "properties": {
                "city": {
                    "type": "string",
                    "description": "The city to get the weather for"
                }
            },
            "required": ["city"]
        });
        let parameters: Schema = serde_json::from_value(schema_value).unwrap();

        let tool_info = ToolInfo {
            tool_type: ToolType::Function,
            function: ToolFunctionInfo {
                name: "get_weather".to_string(),
                description: "Get the weather in a given city".to_string(),
                parameters,
            },
        };

        // Verify structure
        let json = serde_json::to_value(&tool_info).unwrap();
        assert_eq!(json["type"], "function"); // lowercase, not "Function"
        assert_eq!(json["function"]["name"], "get_weather");
        assert_eq!(json["function"]["description"], "Get the weather in a given city");
    }
}
