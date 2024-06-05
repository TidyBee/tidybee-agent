use tracing::error;

#[tokio::main]
async fn main() {
    if let Err(err) = tidybee_agent::run().await {
        error!("Error: {}", err);
    }
}
