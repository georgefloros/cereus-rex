// src/tools/code_context.rs
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug)]
pub struct CodeContextRequest {
    pub operation: String, // "get", "set", "update", "clear"
    pub key: String,
    pub value: Option<serde_json::Value>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CodeContextResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

// For the non-threadsafe version, we'll implement a simpler version that just returns mock responses
// since we can't safely modify state without synchronization
pub struct CodeContextTool;

impl CodeContextTool {
    pub fn new() -> Self {
        Self
    }

    pub async fn execute(&self, request: CodeContextRequest) -> Result<CodeContextResponse, Box<dyn std::error::Error>> {
        // For the non-threadsafe version, just return mock responses
        match request.operation.as_str() {
            "get" => Ok(CodeContextResponse {
                success: true,
                message: "Context retrieved successfully".to_string(),
                data: Some(serde_json::json!({
                    "key": &request.key,
                    "value": "mock_value",
                    "metadata": {}
                })),
            }),
            "set" => {
                if request.value.is_some() {
                    Ok(CodeContextResponse {
                        success: true,
                        message: "Context set successfully".to_string(),
                        data: Some(serde_json::json!({
                            "key": &request.key
                        })),
                    })
                } else {
                    Ok(CodeContextResponse {
                        success: false,
                        message: "Value required for set operation".to_string(),
                        data: None,
                    })
                }
            },
            "update" => {
                if request.value.is_some() {
                    Ok(CodeContextResponse {
                        success: true,
                        message: "Context updated successfully".to_string(),
                        data: Some(serde_json::json!({
                            "key": &request.key
                        })),
                    })
                } else {
                    Ok(CodeContextResponse {
                        success: false,
                        message: "Value required for update operation".to_string(),
                        data: None,
                    })
                }
            },
            "clear" => Ok(CodeContextResponse {
                success: true,
                message: "Context cleared successfully".to_string(),
                data: Some(serde_json::json!({
                    "key": &request.key
                })),
            }),
            _ => Ok(CodeContextResponse {
                success: false,
                message: format!("Unsupported operation: {}", request.operation),
                data: None,
            }),
        }
    }
}

// In a real implementation, this would likely use thread-safe storage
// For now, we'll implement a thread-safe version using Arc<Mutex<...>>
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct ThreadsafeCodeContextTool {
    context_store: Arc<RwLock<HashMap<String, (serde_json::Value, HashMap<String, serde_json::Value>)>>>,
}

impl ThreadsafeCodeContextTool {
    pub fn new() -> Self {
        Self {
            context_store: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn execute(&self, request: CodeContextRequest) -> Result<CodeContextResponse, Box<dyn std::error::Error>> {
        match request.operation.as_str() {
            "get" => self.get_context(&request.key).await,
            "set" => {
                if let Some(value) = request.value {
                    self.set_context(&request.key, value, request.metadata).await
                } else {
                    Ok(CodeContextResponse {
                        success: false,
                        message: "Value required for set operation".to_string(),
                        data: None,
                    })
                }
            },
            "update" => {
                if let Some(value) = request.value {
                    self.update_context(&request.key, value, request.metadata).await
                } else {
                    Ok(CodeContextResponse {
                        success: false,
                        message: "Value required for update operation".to_string(),
                        data: None,
                    })
                }
            },
            "clear" => self.clear_context(&request.key).await,
            _ => Ok(CodeContextResponse {
                success: false,
                message: format!("Unsupported operation: {}", request.operation),
                data: None,
            }),
        }
    }

    async fn get_context(&self, key: &str) -> Result<CodeContextResponse, Box<dyn std::error::Error>> {
        let store = self.context_store.read().await;
        
        if let Some((value, metadata)) = store.get(key) {
            Ok(CodeContextResponse {
                success: true,
                message: "Context retrieved successfully".to_string(),
                data: Some(serde_json::json!({
                    "key": key,
                    "value": value,
                    "metadata": metadata
                })),
            })
        } else {
            Ok(CodeContextResponse {
                success: false,
                message: format!("Context key not found: {}", key),
                data: None,
            })
        }
    }

    async fn set_context(
        &self,
        key: &str,
        value: serde_json::Value,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<CodeContextResponse, Box<dyn std::error::Error>> {
        let metadata = metadata.unwrap_or_default();
        
        {
            let mut store = self.context_store.write().await;
            store.insert(key.to_string(), (value, metadata));
        }
        
        Ok(CodeContextResponse {
            success: true,
            message: "Context set successfully".to_string(),
            data: Some(serde_json::json!({
                "key": key
            })),
        })
    }

    async fn update_context(
        &self,
        key: &str,
        value: serde_json::Value,
        metadata: Option<HashMap<String, serde_json::Value>>,
    ) -> Result<CodeContextResponse, Box<dyn std::error::Error>> {
        let mut store = self.context_store.write().await;
        
        if let Some((existing_value, existing_metadata)) = store.get(key) {
            // Create updated values
            let updated_value = if value.is_object() && existing_value.is_object() {
                let mut existing_map = existing_value.as_object().unwrap().clone();
                let new_map = value.as_object().unwrap();
                
                for (k, v) in new_map {
                    existing_map.insert(k.clone(), v.clone());
                }
                
                serde_json::Value::Object(existing_map)
            } else {
                value // If not objects, just replace
            };
            
            let updated_metadata = if let Some(new_metadata) = metadata {
                let mut merged_metadata = existing_metadata.clone();
                for (k, v) in new_metadata {
                    merged_metadata.insert(k, v);
                }
                merged_metadata
            } else {
                existing_metadata.clone()
            };
            
            store.insert(key.to_string(), (updated_value, updated_metadata));
            
            Ok(CodeContextResponse {
                success: true,
                message: "Context updated successfully".to_string(),
                data: Some(serde_json::json!({
                    "key": key
                })),
            })
        } else {
            Ok(CodeContextResponse {
                success: false,
                message: format!("Context key not found: {}", key),
                data: None,
            })
        }
    }

    async fn clear_context(&self, key: &str) -> Result<CodeContextResponse, Box<dyn std::error::Error>> {
        let mut store = self.context_store.write().await;
        
        if store.remove(key).is_some() {
            Ok(CodeContextResponse {
                success: true,
                message: "Context cleared successfully".to_string(),
                data: Some(serde_json::json!({
                    "key": key
                })),
            })
        } else {
            Ok(CodeContextResponse {
                success: false,
                message: format!("Context key not found: {}", key),
                data: None,
            })
        }
    }
}