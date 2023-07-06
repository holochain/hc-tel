#[tokio::main(flavor = "multi_thread")]
async fn main() {
    hc_tel::init();

    // adding a metric
    // (but this doesn't get output!)
    hc_tel::metric_counter_f64!("my.counter", 1.0, "yo.yo", "bobbo");

    // for some reason, it doesn't seem to output without a span
    let s = tracing::span!(tracing::Level::TRACE, "span");
    let _g = s.enter();
    tracing::trace!("test trace");
}
