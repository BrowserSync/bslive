use bsnext_dto::external_events::has_output_line_matching;
use bsnext_system::cli::from_args_with_output;
use std::env::current_dir;
use std::path::PathBuf;
use std::process;

fn main() {
    unsafe {
        std::env::set_var("RUST_LIB_BACKTRACE", "0");
    }
    let rt = actix_rt::System::new();
    let code = rt.block_on(async_main());
    process::exit(code)
}

async fn async_main() -> i32 {
    let args = "bslive run --sh 'echo 1' --sh 'echo 2'";
    let words = shell_words::split(args).unwrap();
    let cwd = PathBuf::from(current_dir().unwrap().to_string_lossy().to_string());
    let (r, events) = from_args_with_output(words, cwd).await;
    assert!(has_output_line_matching(&events, "1"));
    assert!(has_output_line_matching(&events, "2"));
    match r {
        Ok(_) => 0,
        Err(_) => 1,
    }
}
