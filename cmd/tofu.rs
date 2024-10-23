use aws_sdk_s3::waiters::bucket_exists;
//use aws_sdk_s3::{Client, Config};
//use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::{config::endpoint, Client, Config};
use minio::s3::{client};
use aws_sdk_s3::config::Credentials;
use rand::{distributions::Alphanumeric, Rng};
use tokio::runtime::Runtime;
use std::error::Error;
use std::thread::sleep;
use tokio;
use aws_sdk_s3::config::Region;

use minio::s3::args::{BucketExistsArgs, MakeBucketArgs};
//use minio::s3::builders::ObjectContent;
use minio::s3::client::ClientBuilder;
use minio::s3::creds::StaticProvider;
use minio::s3::http::BaseUrl;
use std::path::Path;

//use aws_config::meta::region::RegionProviderChain;
use std::{fs::File, io::Write, path::PathBuf, process::exit};

use clap::Parser;
use tracing::trace;

// 生成随机 bucket 名字的函数
fn generate_random_bucket_name() -> String {
    let random_string: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    let random_string = random_string.to_lowercase();  // 赋值给 random_string

    format!("probe-bucket-{}", random_string)
}

// 检查随机 bucket 的权限
pub fn check_bucket_permissions(
    url:&str,
    access_key: &str,
    secret_key: &str,
    region: &str,
) -> Result<bool, Box<dyn Error>> {
    // 创建 AWS 配置和 S3 客户端
    println!("Generated random bucket name:{}", url);
    let credentials = Credentials::new(access_key, secret_key, None, None, "");
    //let credentials = Credentials::;
    let region = Region::new(region.to_string());

    let config = Config::builder()
        .region(region)
        .endpoint_url(url)
        .behavior_version(aws_sdk_s3::config::BehaviorVersion::v2023_11_09())
        .credentials_provider(credentials)
        .build();
    dbg!(config.clone());
    let s3_client = Client::from_conf(config);

    // 生成一个随机的 bucket 名字
    let random_bucket_name = generate_random_bucket_name();
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all() // 启用所有 Tokio 的功能
        .build()?; // 构建运行时

    let head_bucket_result = rt.block_on(s3_client.get_bucket_location().bucket(&random_bucket_name).send());
    match head_bucket_result {
        Ok(_) => {
            println!("Bucket '{}' exists and is accessible.", random_bucket_name);
            Ok(true)
        }
        Err(err) => {
            let err_msg = format!("{:?}", err);
            //println!("err: {}\n", err_msg);
            if err_msg.contains("NoSuchBucket") {
                println!("Bucket '{}' does not exist, access key and secret key are valid.", random_bucket_name);
                Ok(true)
            } else if err_msg.contains("SignatureDoesNotMatch") {
                println!("SignatureDoesNotMatch '{}'.", random_bucket_name);
                Ok(false)
            } else if err_msg.contains("InvalidAccessKeyId") {
                println!("InvalidAccessKeyId '{}': {:?}", random_bucket_name, err);
                Ok(false)
            } else {
                println!("other err {}", err);
                Ok(false)
            }
        }
    }
    //Ok((true))
}




// async fn check() -> Result<(), Box<dyn Error>> {
// 	let access_key = "your_access_key";  // 替换为实际的 access key
// 	let secret_key = "your_secret_key";  // 替换为实际的 secret key
// 	let region = "us-west-1";            // 替换为实际的 S3 region
    
// 	// 从外部调用封装的函数
// 	match check_bucket_permissions(access_key, secret_key, region).await {
//         	Ok(true) => println!("Bucket permissions are valid."),
//         	Ok(false) => println!("Bucket permissions are invalid."),
//         	Err(e) => println!("Error checking bucket permissions: {}", e),
//     	}
//     	Ok(())
// }