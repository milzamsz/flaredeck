use uuid::Uuid;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActorKind {
    DesktopUser,
    CliUser,
    McpClient,
    SystemRecovery,
    TestHarness,
}

#[derive(Debug, Clone)]
pub struct OperationContext {
    pub actor: ActorKind,
    pub correlation_id: String,
}

impl OperationContext {
    pub fn new(actor: ActorKind) -> Self {
        Self {
            actor,
            correlation_id: format!("corr_{}", Uuid::new_v4()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ActorKind, OperationContext};

    #[test]
    fn correlation_id_is_safe_and_unique() {
        let first = OperationContext::new(ActorKind::TestHarness);
        let second = OperationContext::new(ActorKind::TestHarness);
        assert!(first.correlation_id.starts_with("corr_"));
        assert_ne!(first.correlation_id, second.correlation_id);
    }
}
