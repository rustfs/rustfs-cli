use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::{Client, Config};
// use rand::{distributions::Alphanumeric, Rng};
// use tokio::runtime::Runtime;
use std::error::Error;
// use std::thread::sleep;
use aws_sdk_s3::config::Region;
use tokio;

fn main() {
    todo!()
    //check_bucket_permissions("http://127.0.0.1:9000", "12345678", "12345678", "us-east-1");
}

pub fn check_bucket_permissions(
    url: &str,
    access_key: &str,
    secret_key: &str,
    region: &str,
) -> Result<bool, Box<dyn Error>> {
    println!("Using S3 endpoint: {}", url);
    let credentials = Credentials::new(access_key, secret_key, None, None, "");
    let region = Region::new(region.to_string());

    let config = Config::builder()
        .region(region)
        .endpoint_url(url)
        .behavior_version(aws_sdk_s3::config::BehaviorVersion::v2023_11_09())
        .credentials_provider(credentials)
        .build();

    let s3_client = Client::from_conf(config);

    // Generate a random bucket name (optional)
    //let random_bucket_name = generate_random_bucket_name();
    //println!("Generated random bucket name: {}", random_bucket_name);

    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()?;

    let list_buckets_result = rt.block_on(s3_client.list_buckets().send());
    match list_buckets_result {
        Ok(output) => {
            println!("Successfully listed buckets: {:?}", output.buckets);
            Ok(true)
        }
        Err(err) => {
            println!("Error listing buckets: {:?}", err);
            Ok(false)
        }
    }
}
