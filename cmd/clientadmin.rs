use aws_sdk_s3::config::Credentials;
use aws_sigv4::http_request::SignableBody;
use aws_sigv4::http_request::{sign, SignableRequest, SigningParams, SigningSettings};
use aws_sigv4::sign::v4;
use http1;
use reqwest::Client;
use std::time::SystemTime;
//use http0;
use aws_smithy_runtime_api::client::identity::Identity;

//#[cfg(feature = "http1")]
async fn test() -> Result<(), Box<dyn std::error::Error>> {
    // Set up information and settings for the signing
    // You can obtain credentials from `SdkConfig`.
    let identity = Credentials::new("12345678", "12345678", None, None, "").into();
    let signing_settings = SigningSettings::default();
    let signing_params = v4::SigningParams::builder()
        .identity(&identity)
        .region("us-east-1")
        .name("exampleservice")
        .time(SystemTime::now())
        .settings(signing_settings)
        .build()
        .unwrap()
        .into();
    // Convert the HTTP request into a signable request
    let signable_request = SignableRequest::new(
        "GET",
        "http://127.0.0.1:9000/info",
        std::iter::empty(),
        SignableBody::Bytes(&[]),
    )
    .expect("signable request");

    let mut my_req = http::Request::new("...");
    // Sign and then apply the signature to the request
    let (signing_instructions, _signature) = sign(signable_request, &signing_params)?.into_parts();
    signing_instructions.apply_to_request_http1x(&mut my_req);
    let cli = Client::new();

    //cli.execute(my_req.try_into()).await;
    //cli.request(method, url)
    let response = cli.execute(my_req.try_into()?).await?;

    // Print response for verification
    println!("Status: {}", response.status());
    println!("Body: {:?}", response.text().await?);
    //reqwest::Client::
    Ok(())
}

#[tokio::main]
pub async fn main() {
    test().await;
}
