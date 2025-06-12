use crate::{EngineEventHandler};
use std::sync::Arc;
use prometheus::{IntCounterVec, HistogramVec, Registry, opts, register_int_counter_vec, register_histogram_vec};
use stepflow_dto::dto::engine_event::EngineEvent;

pub struct MetricsHook {
    workflow_started: IntCounterVec,
    node_success: IntCounterVec,
    node_failed: IntCounterVec,
    node_duration: HistogramVec,
}

impl MetricsHook {
    pub fn new(registry: &Registry) -> Arc<Self> {
        let workflow_started = register_int_counter_vec!(
            opts!("workflow_started_total", "Total number of workflows started"),
            &["mode"]
        ).unwrap();

        let node_success = register_int_counter_vec!(
            opts!("node_success_total", "Total number of successful node executions"),
            &["state"]
        ).unwrap();

        let node_failed = register_int_counter_vec!(
            opts!("node_failed_total", "Total number of failed node executions"),
            &["state"]
        ).unwrap();

        let node_duration = register_histogram_vec!(
            "node_execution_duration_seconds",
            "Histogram of node execution durations",
            &["state"],
            vec![0.005, 0.01, 0.05, 0.1, 0.5, 1.0, 5.0]
        ).unwrap();

        registry.register(Box::new(workflow_started.clone())).unwrap();
        registry.register(Box::new(node_success.clone())).unwrap();
        registry.register(Box::new(node_failed.clone())).unwrap();
        registry.register(Box::new(node_duration.clone())).unwrap();

        Arc::new(Self {
            workflow_started,
            node_success,
            node_failed,
            node_duration,
        })
    }
}

#[async_trait::async_trait]
impl EngineEventHandler for MetricsHook {
    async fn handle_event(&self, event: EngineEvent) {
        match event {
            EngineEvent::WorkflowStarted { .. } => {
                self.workflow_started.with_label_values(&["deferred"]).inc();
            }
            EngineEvent::NodeSuccess { state_name, .. } => {
                self.node_success.with_label_values(&[&state_name]).inc();
                self.node_duration.with_label_values(&[&state_name]).observe(0.1); // TODO: Use actual duration
            }
            EngineEvent::NodeFailed { state_name, .. } => {
                self.node_failed.with_label_values(&[&state_name]).inc();
            }
            _ => {}
        }
    }
}