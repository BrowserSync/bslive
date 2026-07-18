use bsnext_system::cli::from_args;
use napi::{Env, JsNumber};
use std::env::current_dir;
use std::path::PathBuf;

pub struct AsyncStart {
    pub(crate) args: Vec<String>,
    pub(crate) rx: Option<tokio::sync::oneshot::Receiver<()>>,
}

impl napi::Task for AsyncStart {
    type Output = i32;
    type JsValue = JsNumber;

    fn compute(&mut self) -> napi::Result<Self::Output> {
        let sys = actix_rt::System::new();
        let args = self.args.clone();
        let rx = self.rx.take().expect("must be there");
        let cwd = PathBuf::from(current_dir().unwrap().to_string_lossy().to_string());
        unsafe {
            std::env::set_var("RUST_LIB_BACKTRACE", "0");
        }
        let result = sys.block_on(async move {
            tokio::select! {
                _ = rx => {
                    2
                }
                res = from_args(args, cwd) => {
                    match res {
                        Ok(_) => 0,
                        Err(_) => 1,
                    }
                }
            }
        });
        Ok(result)
    }

    fn resolve(&mut self, env: Env, output: Self::Output) -> napi::Result<Self::JsValue> {
        env.create_int32(output)
    }
}
