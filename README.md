# SES Test Email AWS Lambda

A very simple AWS lambda function which, in response to a POST request of the following format:
```json
{
    "email": "test@example.com"
}
```
sends out a test email to the provided email from the source given in the environment variable `FROM_EMAIL`.

Written in Rust using the official AWS SDK.
To run locally, once you've set the `FROM_EMAIL` environment variable (we support using a `.env` file), set up Amazon SES, and logged in using the AWS CLI, simply use `cargo lambda`.
