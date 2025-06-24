use aws_sdk_s3::config::Region;
use aws_sdk_s3::{Client, Config};
use rand::{distributions::Alphanumeric, Rng};
use std::error::Error;
use tokio;

// 生成随机 bucket 名字的函数
fn generate_random_bucket_name() -> String {
    let random_string: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(10)
        .map(char::from)
        .collect();
    let random_string = random_string.to_lowercase(); // 赋值给 random_string

    format!("probe-bucket-{}", random_string)
}

// 检查随机 bucket 的权限
pub async fn check_bucket_permissions(
    url: &str,
    ak: &str,
    sk: &str,
    region: &str,
) -> Result<bool, Box<dyn Error>> {
    // 创建 AWS 配置和 S3 客户端
    println!("Generated random bucket name:{}", url);
    let s3_client = crate::s3::client::get_s3client_from_para(ak, sk, url, region)?;

    // 生成一个随机的 bucket 名字
    let random_bucket_name = generate_random_bucket_name();
    let head_bucket_result = s3_client
        .get_bucket_location()
        .bucket(&random_bucket_name)
        .send()
        .await;
    match head_bucket_result {
        Ok(_) => {
            println!("Bucket '{}' exists and is accessible.", random_bucket_name);
            Ok(true)
        }
        Err(err) => {
            let err_msg = format!("{:?}", err);
            //println!("err: {}\n", err_msg);
            if err_msg.contains("NoSuchBucket") {
                println!(
                    "Bucket '{}' does not exist, access key and secret key are valid.",
                    random_bucket_name
                );
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
}
