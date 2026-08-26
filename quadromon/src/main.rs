use quadromon::AppContext;

fn main() {
    println!("Starting quadromon...");
    let mut app_ctx = AppContext::new();
    println!(
        "quadromon initialized successfully with {} module(s). Poll interval: {} ms",
        app_ctx.config().modules.len(),
        app_ctx.config().server.poll_interval_ms
    );

    // Clean shutdown of the sensor server upon application completion
    app_ctx.shutdown();
}
