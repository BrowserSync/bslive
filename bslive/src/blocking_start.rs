use bsnext_system::cli::from_args;
use std::env::current_dir;
use std::path::PathBuf;

/// Launch in a blocking way
#[allow(dead_code)]
#[napi]
pub fn start_blocking(args: Vec<String>) -> napi::bindgen_prelude::Result<i32> {
    let sys = actix_rt::System::new();
    let result = sys.block_on(async move {
        let cwd = PathBuf::from(current_dir().unwrap().to_string_lossy().to_string());
        unsafe {
            std::env::set_var("RUST_LIB_BACKTRACE", "0");
        }
        match from_args(args, cwd).await {
            Ok(_) => 0,
            Err(_) => 1,
        }
    });
    Ok(result)
}
