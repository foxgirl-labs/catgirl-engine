use build_info_build::DependencyDepth;
use std::env;

fn main() {
    // Generate build info
    generate_build_info();
}

fn matches_environment_var(key: &str, value: &str) -> bool {
    let environment_var: Result<String, env::VarError> = env::var(key);
    environment_var.is_ok() && environment_var.unwrap() == value
}

fn generate_build_info() {
    let mut depth: DependencyDepth = DependencyDepth::Depth(0);

    // Custom environment variable to speed up writing code
    let rust_analyzer: bool = matches_environment_var("RUST_ANALYZER", "true");
    if rust_analyzer {
        depth = DependencyDepth::None;
    }

    build_info_build::build_script().collect_runtime_dependencies(depth);
}