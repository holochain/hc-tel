use once_cell::sync::Lazy;
use std::collections::HashMap;
use std::sync::Mutex;

/// initialize tracing / telemetry / metrics.
pub fn init() {
    use tracing_subscriber::layer::SubscriberExt;

    // set up to print tracing to stdout
    let tracer = opentelemetry::sdk::export::trace::stdout::new_pipeline()
        .install_simple();
    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);

    // *try* setting up to print metrics to stdout
    let check = opentelemetry::sdk::metrics::processors::factory(
        opentelemetry::sdk::metrics::selectors::simple::inexpensive(),
        opentelemetry::sdk::export::metrics::aggregation::cumulative_temporality_selector(),
    );
    let controller = opentelemetry::sdk::metrics::controllers::basic(check)
        .with_exporter(opentelemetry::sdk::export::metrics::stdout().build().unwrap())
        .build();
    let metrics = tracing_opentelemetry::MetricsLayer::new(controller);

    // build our subscriber
    let subscriber = tracing_subscriber::Registry::default()
        .with(metrics)
        .with(telemetry);
    let _ = tracing::subscriber::set_global_default(subscriber);
}

trait MutexExt<T: ?Sized> {
    fn alock(&self) -> std::sync::MutexGuard<'_, T>;
}

impl<T: ?Sized> MutexExt<T> for Mutex<T> {
    fn alock(&self) -> std::sync::MutexGuard<'_, T> {
        self.lock().unwrap()
    }
}

static METRICS: Lazy<opentelemetry::metrics::Meter> = Lazy::new(|| {
    opentelemetry::global::meter("metrics")
});

static F64_COUNTER_MAP: Lazy<Mutex<HashMap<&'static str, opentelemetry::metrics::Counter<f64>>>> = Lazy::new(|| Mutex::new(HashMap::new()));

#[doc(hidden)]
#[inline]
pub fn __get_f64_counter(name: &'static str) -> opentelemetry::metrics::Counter<f64> {
    use std::collections::hash_map::Entry;
    match F64_COUNTER_MAP.alock().entry(name) {
        Entry::Occupied(e) => {
            e.get().clone()
        }
        Entry::Vacant(e) => {
            e.insert(METRICS.f64_counter(name).init()).clone()
        }
    }
}

#[macro_export]
macro_rules! metric_counter_f64 {
    ($n:literal, $c:literal $(, $k:literal, $v:expr)*) => {{
        // nope, having a span doesn't make the metric print
        // let root = tracing::span!(tracing::Level::TRACE, "app_start", work_units = 2);
        // let _enter = root.enter();

        let g = $crate::__get_f64_counter($n);
        g.add(&opentelemetry::Context::current(), $c, &[
            $(opentelemetry::KeyValue::new($k, $v),)*
        ]);

        // nope, the actual values aren't in any Debug impls
        // println!("{g:?} {:?}", opentelemetry::Context::current());
    }};
}
