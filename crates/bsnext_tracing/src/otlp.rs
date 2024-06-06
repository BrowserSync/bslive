use opentelemetry::{global, KeyValue};
use opentelemetry_sdk::{
    metrics::{
        reader::{DefaultAggregationSelector, DefaultTemporalitySelector},
        MeterProviderBuilder, PeriodicReader, SdkMeterProvider,
    },
    runtime,
    trace::{BatchConfig, RandomIdGenerator, Sampler, Tracer},
    Resource,
};
use opentelemetry_semantic_conventions::{
    resource::{DEPLOYMENT_ENVIRONMENT, SERVICE_NAME, SERVICE_VERSION},
    SCHEMA_URL,
};
use std::fs::File;

use crate::{OtelOption, OutputFormat, WriteOption};
use tracing_opentelemetry::{MetricsLayer, OpenTelemetryLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt, Layer};

// Create a Resource that captures information about the entity for which telemetry is recorded.
fn resource() -> Resource {
    Resource::from_schema_url(
        [
            KeyValue::new(SERVICE_NAME, env!("CARGO_PKG_NAME")),
            KeyValue::new(SERVICE_VERSION, env!("CARGO_PKG_VERSION")),
            KeyValue::new(DEPLOYMENT_ENVIRONMENT, "develop"),
        ],
        SCHEMA_URL,
    )
}

// Construct MeterProvider for MetricsLayer
fn init_meter_provider() -> SdkMeterProvider {
    let exporter = opentelemetry_otlp::new_exporter()
        .tonic()
        .build_metrics_exporter(
            Box::new(DefaultAggregationSelector::new()),
            Box::new(DefaultTemporalitySelector::new()),
        )
        .unwrap();

    let reader = PeriodicReader::builder(exporter, runtime::Tokio)
        .with_interval(std::time::Duration::from_secs(30))
        .build();

    // For debugging in development
    let stdout_reader = PeriodicReader::builder(
        opentelemetry_stdout::MetricsExporter::default(),
        runtime::Tokio,
    )
    .build();

    let meter_provider = MeterProviderBuilder::default()
        .with_resource(resource())
        .with_reader(reader)
        .with_reader(stdout_reader)
        .build();

    global::set_meter_provider(meter_provider.clone());

    meter_provider
}

// Construct Tracer for OpenTelemetryLayer
fn init_tracer() -> Tracer {
    opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_trace_config(
            opentelemetry_sdk::trace::Config::default()
                // Customize sampling strategy
                .with_sampler(Sampler::ParentBased(Box::new(Sampler::TraceIdRatioBased(
                    1.0,
                ))))
                // If export trace to AWS X-Ray, you can use XrayIdGenerator
                .with_id_generator(RandomIdGenerator::default())
                .with_resource(resource()),
        )
        .with_batch_config(BatchConfig::default())
        .with_exporter(opentelemetry_otlp::new_exporter().tonic())
        .install_batch(runtime::Tokio)
        .unwrap()
}

// Initialize tracing-subscriber and return OtelGuard for opentelemetry-related termination processing
pub fn init_tracing_subscriber(
    debug_str: &str,
    format: Option<OutputFormat>,
    write_option: WriteOption,
    otel: OtelOption,
) -> OtelGuard {
    let filter =
        tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| debug_str.into());

    let fmt_layer = match (format.unwrap_or_default(), write_option) {
        (OutputFormat::Json, WriteOption::None) => tracing_subscriber::fmt::layer()
            .without_time()
            .json()
            .with_file(false)
            .boxed(),
        (OutputFormat::Json, WriteOption::File) => {
            let file = File::create("bslive.log").expect("create bslive.log");
            tracing_subscriber::fmt::layer()
                .json()
                .with_ansi(false)
                // todo(alpha): use this example as a way to move this output into the terminal window
                .with_writer(file)
                .boxed()
        }
        (OutputFormat::Normal, WriteOption::None) => tracing_subscriber::fmt::layer()
            .without_time()
            .with_file(false)
            .boxed(),
        (OutputFormat::Normal, WriteOption::File) => {
            let file = File::create("bslive.log").expect("create bslive.log");
            tracing_subscriber::fmt::layer()
                .with_ansi(false)
                .with_writer(file)
                .boxed()
        }
    };

    if otel == OtelOption::On {
        let meter_provider = init_meter_provider();
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .with(MetricsLayer::new(meter_provider.clone()))
            .with(OpenTelemetryLayer::new(init_tracer()))
            .init();
        OtelGuard {
            meter_provider: Some(meter_provider),
        }
    } else {
        tracing_subscriber::registry()
            .with(filter)
            .with(fmt_layer)
            .init();
        OtelGuard {
            meter_provider: None,
        }
    }
}

pub struct OtelGuard {
    pub meter_provider: Option<SdkMeterProvider>,
}

impl Drop for OtelGuard {
    fn drop(&mut self) {
        match &self.meter_provider {
            None => {
                // nothing to do here
            }
            Some(provider) => {
                if let Err(err) = provider.shutdown() {
                    eprintln!("{err:?}");
                }
                opentelemetry::global::shutdown_tracer_provider();
            }
        }
    }
}
