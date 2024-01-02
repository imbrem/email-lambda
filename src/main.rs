use aws_config::BehaviorVersion;
use aws_sdk_sesv2::types::{Body as EmailBody, Content, Destination, EmailContent, Message};
use lambda_http::{http, run, service_fn, Body, Error, Request, Response};
use serde_json::Value;

// Process an HTTP request
async fn function_handler(
    event: Request,
    from_email: &str,
    client: &aws_sdk_sesv2::Client,
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

    let destination = Destination::builder().to_addresses(email).build();
    let content = EmailContent::builder()
        .simple(
            Message::builder()
                .subject(Content::builder().data("SES Test Email").build()?)
                .body(
                    EmailBody::builder()
                        .html(
                            Content::builder()
                                .data(
                                    "<h1>SES Test Email</h1><p>This email was sent from Rust!</p>",
                                )
                                .build()?,
                        )
                        .build(),
                )
                .build(),
        )
        .build();

    client
        .send_email()
        .from_email_address(from_email)
        .destination(destination)
        .content(content)
        .send().await?;

    // Return success on properly processed webhook
    let message = format!("Successfully handled webhook!");
    let resp = Response::builder()
        .status(200)
        .header("content-type", "text/html")
        .body(message.into())?;
    Ok(resp)
}

#[tokio::main]
async fn main() -> Result<(), Error> {
    dotenv::dotenv()?;
    let from_email = std::env::var("FROM_EMAIL")?;

    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        // disable printing the name of the module in every log line.
        .with_target(false)
        // disabling time is handy because CloudWatch will add the ingestion time.
        .without_time()
        .init();

    let sdk_config = aws_config::load_defaults(BehaviorVersion::latest()).await;
    let client = aws_sdk_sesv2::Client::new(&sdk_config);

    run(service_fn(|event| {
        function_handler(event, &from_email, &client)
    }))
    .await
}
