pub mod skill_loader;
pub mod state;
pub mod config;
pub mod types;

// Re-exports para facilitar el uso
pub use skill_loader::{load_agent_skill, load_skill};
pub use state::{OpStatus, OpType, Operation, OperationStore};
pub use config::FactoryConfig;
pub use types::{ChannelConfig, Script, VideoAsset};
