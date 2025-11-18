// src/agents/sync.rs
use crate::mcp::types::AgentSession;
use dashmap::DashMap;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Represents the coordination state between multiple agents
#[derive(Debug, Clone)]
pub struct AgentCoordinator {
    pub agents: Arc<DashMap<String, AgentSession>>,
    pub active_sessions: Arc<DashMap<String, AgentSession>>,
    pub shared_context: Arc<RwLock<SharedContext>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SharedContext {
    pub id: String,
    pub data: std::collections::HashMap<String, serde_json::Value>,
    pub updated_at: u64,
    pub owner: Option<String>, // Agent ID that owns this context
    pub collaborators: Vec<String>, // Agent IDs that can access this context
}

impl AgentCoordinator {
    pub fn new() -> Self {
        Self {
            agents: Arc::new(DashMap::new()),
            active_sessions: Arc::new(DashMap::new()),
            shared_context: Arc::new(RwLock::new(SharedContext {
                id: Uuid::new_v4().to_string(),
                data: std::collections::HashMap::new(),
                updated_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
                owner: None,
                collaborators: vec![],
            })),
        }
    }

    /// Register a new agent
    pub async fn register_agent(&self, mut agent: AgentSession) -> String {
        let agent_id = Uuid::new_v4().to_string();
        agent.id = agent_id.clone();
        agent.last_seen = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        self.agents.insert(agent_id.clone(), agent.clone());
        self.active_sessions.insert(agent_id.clone(), agent);

        agent_id
    }

    /// Update agent presence
    pub async fn update_agent_presence(&self, agent_id: &str) -> bool {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        if let Some(mut agent) = self.agents.get_mut(agent_id) {
            agent.last_seen = now;
            if let Some(mut active_agent) = self.active_sessions.get_mut(agent_id) {
                active_agent.last_seen = now;
                true
            } else {
                // Agent not in active sessions, add it
                self.active_sessions.insert(agent_id.to_string(), agent.clone());
                true
            }
        } else {
            false
        }
    }

    /// Remove agent from active sessions
    pub async fn deregister_agent(&self, agent_id: &str) -> bool {
        self.active_sessions.remove(agent_id).is_some()
    }

    /// Get all active agents
    pub async fn get_active_agents(&self) -> Vec<AgentSession> {
        self.active_sessions
            .iter()
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Update shared context
    pub async fn update_shared_context(&self, context_updates: std::collections::HashMap<String, serde_json::Value>) {
        let mut shared_context = self.shared_context.write().await;
        shared_context.data.extend(context_updates);
        shared_context.updated_at = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
    }

    /// Get shared context
    pub async fn get_shared_context(&self) -> SharedContext {
        self.shared_context.read().await.clone()
    }

    /// Add collaborator to shared context
    pub async fn add_collaborator(&self, agent_id: String) -> bool {
        let mut shared_context = self.shared_context.write().await;
        if !shared_context.collaborators.contains(&agent_id) {
            shared_context.collaborators.push(agent_id);
            shared_context.updated_at = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            true
        } else {
            false
        }
    }

    /// Remove collaborator from shared context
    pub async fn remove_collaborator(&self, agent_id: &str) -> bool {
        let mut shared_context = self.shared_context.write().await;
        let initial_len = shared_context.collaborators.len();
        shared_context.collaborators.retain(|id| id != agent_id);
        let final_len = shared_context.collaborators.len();
        final_len != initial_len
    }
}

impl Default for AgentCoordinator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_agent_registration() {
        let coordinator = AgentCoordinator::new();
        let agent = AgentSession {
            id: "test".to_string(),
            name: "Test Agent".to_string(),
            capabilities: vec!["search".to_string()],
            last_seen: 0,
            context: crate::mcp::types::AgentContext {
                id: "context".to_string(),
                name: "Test Context".to_string(),
                data: std::collections::HashMap::new(),
                created_at: 0,
                updated_at: 0,
            },
        };

        let agent_id = coordinator.register_agent(agent).await;
        assert!(!agent_id.is_empty());

        let active_agents = coordinator.get_active_agents().await;
        assert_eq!(active_agents.len(), 1);
    }

    #[tokio::test]
    async fn test_shared_context() {
        let coordinator = AgentCoordinator::new();
        
        let mut updates = std::collections::HashMap::new();
        updates.insert("test_key".to_string(), serde_json::Value::String("test_value".to_string()));
        
        coordinator.update_shared_context(updates).await;
        
        let context = coordinator.get_shared_context().await;
        assert!(context.data.contains_key("test_key"));
        assert_eq!(context.data.get("test_key").unwrap().as_str().unwrap(), "test_value");
    }
}