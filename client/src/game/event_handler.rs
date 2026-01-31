use crate::types::contract::events::GameEvent;
use async_trait::async_trait;

#[async_trait]
pub trait EventHandler: Send + Sync {
    async fn handle_event(&self, event: GameEvent);
}