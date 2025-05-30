use stepflow_dsl::WorkflowDSL;
use std::fs;
use std::path::Path;

fn load_json(path: &Path) -> WorkflowDSL {
    let content = fs::read_to_string(path).expect("failed to read json file");
    serde_json::from_str(&content).expect("failed to parse json")
}

fn load_yaml(path: &Path) -> WorkflowDSL {
    let content = fs::read_to_string(path).expect("failed to read yaml file");
    serde_yaml::from_str(&content).expect("failed to parse yaml")
}

#[test]
fn test_task_example() {
    let json = load_json(Path::new("examples/task/task.json"));
    println!("✅ task.json loaded: startAt = {}", json.start_at);

    let yaml = load_yaml(Path::new("examples/task/task.yaml"));
    println!("✅ task.yaml loaded: startAt = {}", yaml.start_at);

    assert_eq!(json.start_at, "MyTask");
    assert_eq!(yaml.start_at, "MyTask");
}

#[test]
fn test_choice_example() {
    let json = load_json(Path::new("examples/choice/choice.json"));
    let yaml = load_yaml(Path::new("examples/choice/choice.yaml"));
    assert_eq!(json.start_at, "Choose");
    assert_eq!(yaml.start_at, "Choose");
}

#[test]
fn test_parallel_example() {
    let json = load_json(Path::new("examples/parallel/parallel.json"));
    let yaml = load_yaml(Path::new("examples/parallel/parallel.yaml"));
    assert_eq!(json.start_at, "RunParallel");
    assert_eq!(yaml.start_at, "RunParallel");
}

#[test]
fn test_map_example() {
    let json = load_json(Path::new("examples/map/map.json"));
    let yaml = load_yaml(Path::new("examples/map/map.yaml"));
    assert_eq!(json.start_at, "MapItems");
    assert_eq!(yaml.start_at, "MapItems");
}

#[test]
fn test_fail_example() {
    let json = load_json(Path::new("examples/fail/fail.json"));
    let yaml = load_yaml(Path::new("examples/fail/fail.yaml"));
    assert_eq!(json.start_at, "FailNow");
    assert_eq!(yaml.start_at, "FailNow");
}

#[test]
fn test_full_example() {
    let json = load_json(Path::new("examples/full/full.json"));
    assert_eq!(json.start_at, "PrepareTask");
}