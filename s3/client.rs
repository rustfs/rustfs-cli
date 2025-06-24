use crate::cmd::aliasremove::get_alias;
use aws_sdk_s3::config::BehaviorVersion;
use aws_sdk_s3::config::Credentials;
use aws_sdk_s3::config::Region;
use aws_sdk_s3::{Client as S3Client, Config};
use std::error::Error;

pub fn get_s3client_from_alias(alias: &str) -> Result<S3Client, Box<dyn Error>> {
    let conf = get_alias(alias)?; // Propagate error from get_alias directly
    println!("get config {} suc", alias);
    get_s3client_from_para(&conf.access_key, &conf.secret_key, &conf.url, "us-east-1")
}

pub fn get_s3client_from_para(
    ak: &str,
    sk: &str,
    url: &str,
    _region: &str,
) -> Result<S3Client, Box<dyn Error>> {
    let credentials = Credentials::new(ak, sk, None, None, "");
    let region = Region::new("us-east-1".to_string());

    let config = Config::builder()
        .region(region)
        .endpoint_url(url.to_string())
        .credentials_provider(credentials)
        .behavior_version(BehaviorVersion::latest()) // Adjust as necessary
        .build();
    Ok(S3Client::from_conf(config))
}

// pub fn check_bucket_permissions(
//     url: &str,
//     access_key: &str,
//     secret_key: &str,
//     region: &str,
// ) -> Result<bool, Box<dyn Error>> {
//     println!("Using S3 endpoint: {}", url);
//     let s3_client = get_s3client_from_para(access_key, secret_key, url, "us-east-1")?;

//     let rt = tokio::runtime::Builder::new_current_thread()
//         .enable_all()
//         .build()?;

//     let list_buckets_result = rt.block_on(s3_client.list_buckets().send());
//     match list_buckets_result {
//         Ok(output) => {
//             println!("Successfully listed buckets: {:?}", output.buckets);
//             Ok(true)
//         }
//         Err(err) => {
//             println!("Error listing buckets: {:?}", err);
//             Ok(false)
//         }
//     }
// }

// struct Client;

// impl Client {
//     async fn put(
//         &self,
//         reader: &mut (dyn tokio::io::AsyncRead + Unpin),
//         size: u64,
//         progress: Option<&mut (dyn tokio::io::AsyncRead + Unpin)>,
//         opts: PutOptions,
//     ) -> IoResult<u64> {
//         // Dummy implementation for data upload logic
//         let mut buffer = vec![0; size as usize];
//         let mut total_read = 0;

//         while total_read < size {
//             let read_size = reader.read(&mut buffer[total_read as usize..]).await?;
//             if read_size == 0 {
//                 break;
//             }
//             total_read += read_size as u64;
//         }

//         // In a real scenario, progress and options would be used in data upload logic
//         Ok(total_read)
//     }
// }

// async fn put_target_stream(
//     alias: &str,
//     url_str: &str,
//     mode: Option<&str>,
//     until: Option<&str>,
//     legal_hold: Option<&str>,
//     mut reader: &mut (dyn tokio::io::AsyncRead + Unpin),
//     size: u64,
//     mut progress: Option<&mut (dyn tokio::io::AsyncRead + Unpin)>,
//     mut opts: PutOptions,
// ) -> Result<u64, String> {
//     let target_client = match new_client_from_alias(alias, url_str).await {
//         Ok(client) => client,
//         Err(err) => return Err(format!("Error creating client from alias: {}", err)),
//     };

//     if let Some(mode_str) = mode {
//         opts.metadata
//             .insert("AmzObjectLockMode".to_string(), mode_str.to_string());
//     }
//     if let Some(until_str) = until {
//         opts.metadata.insert(
//             "AmzObjectLockRetainUntilDate".to_string(),
//             until_str.to_string(),
//         );
//     }
//     if let Some(legal_hold_str) = legal_hold {
//         opts.metadata.insert(
//             "AmzObjectLockLegalHold".to_string(),
//             legal_hold_str.to_string(),
//         );
//     }

//     match target_client
//         .put(&mut reader, size, progress.as_deref_mut(), opts)
//         .await
//     {
//         Ok(bytes_written) => Ok(bytes_written),
//         Err(err) => Err(format!("Error uploading data: {}", err)),
//     }
// }
