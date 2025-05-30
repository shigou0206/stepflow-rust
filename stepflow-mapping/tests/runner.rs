//! 批量回归测试：读取 examples/{case}/mapping.yaml + input.json
//! 针对 expected_output.json 或 expected_error.txt 进行断言

use std::{fs, path::Path};

use stepflow_mapping::{MappingDSL, MappingEngine};
use serde_json::Value;

/// 读取目录下的 mapping.yaml、input.json
fn load_case(dir: &Path) -> (MappingDSL, Value) {
    let yaml   = fs::read_to_string(dir.join("mapping.yaml")).unwrap();
    let input  = fs::read_to_string(dir.join("input.json")).unwrap();

    let dsl: MappingDSL = serde_yaml::from_str(&yaml).unwrap();
    let input_json:  Value = serde_json::from_str(&input).unwrap();

    (dsl, input_json)
}

#[test]
fn run_all_examples() {
    let examples_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples");

    for entry in fs::read_dir(&examples_dir).unwrap() {
        let path = entry.unwrap().path();
        if !path.is_dir() {
            continue;
        }
        let case_name = path.file_name().unwrap().to_string_lossy();
        println!("running case: {}", case_name);

        let (dsl, input_json) = load_case(&path);

        // 如果存在 expected_error.txt，则应返回错误
        let err_file = path.join("expected_error.txt");
        if err_file.exists() {
            let expected_err = fs::read_to_string(&err_file).unwrap();
            let result = MappingEngine::apply(dsl, &input_json);
            assert!(
                result.is_err(),
                "case `{}` expected error but got Ok",
                case_name
            );
            let err_msg = result.err().unwrap().to_string();
            assert_eq!(
                err_msg.trim(),
                expected_err.trim(),
                "case `{}` failed: error mismatch",
                case_name
            );
        } else {
            // 否则对 expected_output.json 做比对
            let expect = fs::read_to_string(path.join("expected_output.json")).unwrap();
            let expect_json: Value = serde_json::from_str(&expect).unwrap();
            let output = MappingEngine::apply(dsl, &input_json)
                .unwrap_or_else(|e| panic!("case `{}` unexpected error: {}", case_name, e));
            assert_eq!(
                output,
                expect_json,
                "case `{}` failed: output != expected_output.json",
                case_name
            );
        }
    }
}