pub mod engine;

pub fn register_all_builtin_errors() {
    engine::register_engine_errors();
    // dsl::register_dsl_errors();
    // ...
}