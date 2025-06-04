pub mod workflow;
pub mod event;
pub mod state;
pub mod activity;
pub mod timer;
pub mod template;
pub mod visibility;
pub mod queue;

// Re-export all traits
pub use workflow::WorkflowStorage;
pub use event::EventStorage;
pub use state::StateStorage;
pub use activity::ActivityStorage;
pub use timer::TimerStorage;
pub use template::TemplateStorage;
pub use visibility::VisibilityStorage;
pub use queue::QueueStorage;

// Storage trait that combines all storage traits
pub trait Storage: 
    WorkflowStorage + 
    EventStorage + 
    StateStorage + 
    ActivityStorage + 
    TimerStorage + 
    TemplateStorage + 
    VisibilityStorage + 
    QueueStorage + 
    Send + 
    Sync 
{} 