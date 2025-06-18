use std::collections::HashMap;
use std::sync::Mutex;
use once_cell::sync::Lazy;

#[derive(Debug, Clone)]
pub struct ErrorDescriptor {
    pub name: &'static str,         // error_type，如 "HttpTimeout"
    pub category: &'static str,     // "Tool" / "Engine" / ...
    pub description: &'static str,  // 用户可见说明
}

pub static ERROR_REGISTRY: Lazy<Mutex<HashMap<&'static str, ErrorDescriptor>>> = Lazy::new(|| {
    Mutex::new(HashMap::new())
});

pub fn register_error(name: &'static str, descriptor: ErrorDescriptor) {
    let mut reg = ERROR_REGISTRY.lock().unwrap();
    reg.insert(name, descriptor);
}

pub fn get_error_descriptor(name: &str) -> Option<ErrorDescriptor> {
    ERROR_REGISTRY.lock().unwrap().get(name).cloned()
}