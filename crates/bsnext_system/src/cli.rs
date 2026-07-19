use crate::args::{Args, SubCommands};
use crate::start;
use crate::start::start_kind::StartKind;
use crate::start::stdout_channel;
use bsnext_dto::any_event::AnyEvent;
use bsnext_output::OutputWriters;
use bsnext_tracing::{
    init_tracing, init_tracing_with_otel, LineNumberOption, OutputFormat, WriteOption,
};
use clap::Parser;
use std::ffi::OsString;
use std::future::Future;
use std::path::PathBuf;
use tokio::sync::mpsc;
use tokio::sync::mpsc::Sender;
use tracing::debug_span;

/// The typical lifecycle when ran from a CLI environment
pub async fn from_args<I, T>(itr: I, cwd: PathBuf) -> Result<(), anyhow::Error>
where
    I: IntoIterator<Item = T> + std::fmt::Debug,
    T: Into<OsString> + Clone,
{
    let args = Args::parse_from(itr);

    let logging = *args.logging();
    let write_log_opt = if logging.write_log {
        WriteOption::File
    } else {
        WriteOption::None
    };

    let line_opts = if logging.filenames {
        LineNumberOption::FileAndLineNumber
    } else {
        LineNumberOption::None
    };

    let format = args.format();
    let tracing_guard = if logging.otel {
        init_tracing_with_otel(
            logging.log_level,
            logging.log_http.unwrap_or_default(),
            format,
            write_log_opt,
            line_opts,
        )
    } else {
        init_tracing(
            logging.log_level,
            logging.log_http.unwrap_or_default(),
            format,
            write_log_opt,
            line_opts,
        )
    };

    let format_clone = format;

    let writer = match format_clone {
        OutputFormat::Tui => OutputWriters::Pretty,
        OutputFormat::Normal => OutputWriters::Pretty,
        OutputFormat::Json => OutputWriters::Json,
    };

    let _guard = debug_span!("parent").entered();
    let (sender, fut) = stdout_channel(writer);
    let result = async_init(args, cwd, (sender, fut)).await;
    drop(_guard);
    drop(tracing_guard);
    result
}

/// a way of running that will collect events and not exit until the program exits naturally
pub async fn from_args_with_buffered_output<I, T>(
    itr: I,
    cwd: PathBuf,
) -> (anyhow::Result<()>, Vec<AnyEvent>)
where
    I: IntoIterator<Item = T> + std::fmt::Debug,
    T: Into<OsString> + Clone,
{
    let args = Args::parse_from(itr);

    let ready_future = futures::future::pending();
    let (events_sender, mut events_receiver) = mpsc::channel::<AnyEvent>(100);

    let result = async_init(args, cwd, (events_sender, ready_future)).await;

    // now consume all the events
    let mut events: Vec<AnyEvent> = Vec::new();
    let len = events_receiver.len();

    tracing::debug!("will try to collect {len} events");
    let count = events_receiver.recv_many(&mut events, len).await;

    tracing::debug!("did collect {count} events");
    if count != len {
        tracing::error!(
            "collected events didn't match expectation, expected: {len} actual: {count}"
        );
    }
    (result, events)
}

async fn async_init(
    args: Args,
    cwd: PathBuf,
    (sender, fut): (Sender<AnyEvent>, impl Future<Output = ()> + 'static),
) -> Result<(), anyhow::Error> {
    let fs_opts = args.fs_opts.clone();
    let input_opts = args.input_opts.clone();
    match args.command() {
        SubCommands::Start(start) => {
            let start_kind = start.as_start_kind(&fs_opts, &input_opts);
            start_wrapper(start_kind, cwd, (sender, fut)).await
        }
        SubCommands::Watch(watch) => {
            let start_kind = watch.as_start_kind(&fs_opts, &input_opts);
            start_wrapper(start_kind, cwd, (sender, fut)).await
        }
        SubCommands::Run(run) => {
            let start_kind = run.as_start_kind(&fs_opts, &input_opts);
            start_wrapper(start_kind, cwd, (sender, fut)).await
        }
    }
}

async fn start_wrapper(
    start_kind: StartKind,
    cwd: PathBuf,
    (sender, fut): (Sender<AnyEvent>, impl Future<Output = ()> + 'static),
) -> anyhow::Result<()> {
    let system_handle = actix_rt::spawn(start::with_sender(cwd, start_kind, sender.clone()));
    let channel_handle = actix_rt::spawn(fut);
    let output = tokio::select! {
        r = system_handle => {
            match r {
                Ok(Ok(..)) => Ok(()),
                Ok(Err(err)) => Err(anyhow::anyhow!("1{}", err)),
                Err(e) => Err(anyhow::anyhow!("2{}", e))
            }
        }
        r = channel_handle => {
            match r {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow::anyhow!("3{}", e))
            }
        }
    };
    output
}
