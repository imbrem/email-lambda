use aws_config::BehaviorVersion;
use lambda_http::{http, run, service_fn, Body, Error, Request, Response};
use serde_json::Value;

// Process an HTTP request
async fn function_handler(
    event: Request,
    _client: &aws_sdk_sesv2::Client,
) -> Result<Response<Body>, Error> {
    // We only support POST requests; otherwise, return a 405 Method Not Allowed
    if event.method() != http::Method::POST {
        return Ok(Response::builder()
            .status(http::StatusCode::METHOD_NOT_ALLOWED)
            .body("Method Not Allowed".into())?);
    }

    // Parse the body as JSON; on failure, return a 400 Bad Request
    let Ok(body) = serde_json::from_slice::<Value>(event.body()) else {
        return Ok(Response::builder()
            .status(http::StatusCode::BAD_REQUEST)
            .body("Invalid JSON format in request body".into())?);
    };

    // Get the email key; on failure, return a 400 Bad Request
    let email = match body.get("email") {
        None => {
            return Ok(Response::builder()
                .status(http::StatusCode::BAD_REQUEST)
                .body("JSON object must contain the key \"email\"".into())?);
        }
        Some(Value::String(email)) => email,
        Some(_) => {
            return Ok(Response::builder()
                .status(http::StatusCode::BAD_REQUEST)
                .body("JSON object must contain a string value for the key \"email\"".into())?);
        }
    };

    // Return success on properly processed webhook
    let message = format!("Webhook got email: {email}");
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(message.into())?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = aws_sdk_sesv2::Client::new(&config);

    run(service_fn(|event| function_handler(event, &client))).await
}
