use std::env;

fn main() {
    // Allows to show relevant environment variables for debugging purpose
    print_env();

    let target = env::var("TARGET").unwrap_or_default();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();
    let target_family = env::var("CARGO_CFG_TARGET_FAMILY").unwrap_or_default();
    let out_dir = env::var("OUT_DIR").unwrap();

    let mut b = freertos_cargo_build::Builder::new();

    b.freertos("freertos-kernel");

    b.freertos_config(".");

    b.compile().unwrap_or_else(|e| panic!("{}", e.to_string()));
}

/// Print relevant environment variables
fn print_env() {
    let env_keys = ["TARGET", "OUT_DIR", "HOST"];
    env::vars().for_each(|(key, val)| {
        if key.starts_with("CARGO") {
            println!("cargo:warning={key}={val}");
        } else {
            // println!("cargo:warning={}={}", key, val);
        }
    });
}
