use aws_sdk_s3::{
    primitives::ByteStream,
    types::{CompletedMultipartUpload, CompletedPart},
};
use clap;
use std::{path::Path, result::Result::Ok, sync::Arc};
use tokio::{fs::File, sync::Mutex};

use indicatif::{ProgressBar, ProgressStyle};
use std::error::Error;
use tokio::io::AsyncReadExt;
const CHUNK_SIZE: usize = 64 * 1024 * 1024; // 8 MB

#[derive(clap::Args, Debug)]
pub struct PutOptions {
    #[arg(help = "alias/bucket   (rustfs/bucketxyz)")]
    pub src: String,

    pub target: String,

    #[arg(
        long = "enc-c",
        help = "Encrypt/decrypt objects using client-provided keys. Formats: RawBase64 or Hex."
    )]
    pub enc_c: Option<String>,

    #[arg(
        long = "enc-kms",
        help = "Encrypt/decrypt objects using server-side encryption keys."
    )]
    pub enc_kms: Option<String>,

    #[arg(
        long = "enc-s3",
        help = "Encrypt/decrypt objects using server-side default keys and configurations."
    )]
    pub enc_s3: Option<String>,

    #[arg(
        long = "config-dir",
        short = 'C',
        default_value = "/home/ldy/.mc",
        help = "Path to configuration folder"
    )]
    pub config_dir: String,

    #[arg(long = "quiet", short = 'q', help = "Disable progress bar display")]
    pub quiet: bool,

    #[arg(
        long = "disable-pager",
        help = "Disable mc internal pager and print to raw stdout"
    )]
    pub disable_pager: bool,

    #[arg(long = "no-color", help = "Disable color theme")]
    pub no_color: bool,

    #[arg(long = "json", help = "Enable JSON lines formatted output")]
    pub json: bool,

    #[arg(long = "debug", help = "Enable debug output")]
    pub debug: bool,

    #[arg(
        long = "resolve",
        help = "Resolve HOST[:PORT] to an IP address. Example: minio.local:9000=10.10.75.1"
    )]
    pub resolve: Option<String>,

    #[arg(long = "insecure", help = "Disable SSL certificate verification")]
    pub insecure: bool,

    #[arg(
        long = "limit-upload",
        help = "Limit upload rate in KiB/s, MiB/s, GiB/s"
    )]
    pub limit_upload: Option<String>,

    #[arg(
        long = "limit-download",
        help = "Limit download rate in KiB/s, MiB/s, GiB/s"
    )]
    pub limit_download: Option<String>,

    #[arg(
        long = "checksum",
        help = "Add checksum to uploaded object. Values: MD5, CRC32, CRC32C, SHA1 or SHA256"
    )]
    pub checksum: Option<String>,

    #[arg(
        long = "parallel",
        short = 'P',
        default_value_t = 4,
        help = "Upload number of parts in parallel"
    )]
    pub parallel: u8,

    #[arg(
        long = "part-size",
        short = 's',
        default_value = "16MiB",
        help = "Size of each part"
    )]
    pub part_size: String,

    #[arg(long = "disable-multipart", help = "Disable multipart upload feature")]
    pub disable_multipart: bool,
}

pub async fn handle_put_command(opt: &PutOptions) -> Result<(), Box<dyn std::error::Error>> {
    put(opt).await
}

fn split_first_part(input: &str) -> (&str, &str) {
    let mut parts = input.splitn(2, '/');
    let first_part = parts.next().unwrap_or(""); // 获取 "a1"
    let rest_part = parts.next().unwrap_or(""); // 获取 "a2/a3/a4"

    (first_part, rest_part)
}

fn generate_s3_key(src: &str, target: &str) -> Result<(String, String, String), String> {
    let file_path = Path::new(src);
    let target_path = Path::new(target);

    // Split the target path into components
    let mut target_components = target_path.iter();

    // Extract the alias
    let alias = target_components
        .next()
        .ok_or_else(|| {
            "Target path should have at least two components: alias and bucket".to_string()
        })?
        .to_string_lossy()
        .into_owned();

    // Extract the bucket
    let bucket = target_components
        .next()
        .ok_or_else(|| {
            "Target path should have at least two components: alias and bucket".to_string()
        })?
        .to_string_lossy()
        .into_owned();

    // Check if either alias or bucket is empty
    if alias.is_empty() || bucket.is_empty() {
        return Err("Alias or bucket cannot be empty".to_string());
    }

    // Collect the remaining components to form the key prefix
    let mut s3_key = std::path::PathBuf::new();
    for component in target_components {
        s3_key.push(component);
    }

    // Determine if we should append the file name to the key
    if s3_key.as_os_str().is_empty() {
        let file_name = file_path
            .file_name()
            .ok_or_else(|| "Source path should have a file name".to_string())?
            .to_string_lossy();
        s3_key.push(file_name.to_string());
    } else if target.ends_with('/') {
        // If the target ends with '/', append the file name from the source path
        let file_name = file_path
            .file_name()
            .ok_or_else(|| "Source path should have a file name".to_string())?
            .to_string_lossy();
        s3_key.push(file_name.to_string());
    }

    // Convert the PathBuf to a string
    let key = s3_key.to_string_lossy().into_owned();

    Ok((alias, bucket, key))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_target_with_trailing_slash() {
        let src = "./a.out";
        let target = "alias/bucket/dir1/dir2/dir3/";
        let result = generate_s3_key(src, target).expect("Failed to generate S3 key");

        assert_eq!(result.0, "alias");
        assert_eq!(result.1, "bucket");
        assert_eq!(result.2, "dir1/dir2/dir3/a.out");
    }

    #[test]
    fn test_valid_target_without_trailing_slash() {
        let src = "./a.out";
        let target = "alias/bucket/dir1/dir2/dir3";
        let result = generate_s3_key(src, target).expect("Failed to generate S3 key");

        assert_eq!(result.0, "alias");
        assert_eq!(result.1, "bucket");
        assert_eq!(result.2, "dir1/dir2/dir3");
    }

    #[test]
    fn test_only_alias_and_bucket() {
        let src = "./a.out";
        let target = "alias/bucket";
        let result = generate_s3_key(src, target).expect("Failed to generate S3 key");

        assert_eq!(result.0, "alias");
        assert_eq!(result.1, "bucket");
        assert_eq!(result.2, "a.out");
    }

    #[test]
    fn test_missing_alias_or_bucket() {
        let src = "./a.out";
        let target = "alias/";
        let result = generate_s3_key(src, target);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Alias or bucket cannot be empty"
        );
    }

    #[test]
    fn test_invalid_source_path() {
        let src = "./nonexistent_file";
        let target = "alias/bucket/dir1/";
        let result = generate_s3_key(src, target);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Source path should have a file name"
        );
    }

    #[test]
    fn test_empty_target_path() {
        let src = "./a.out";
        let target = "";
        let result = generate_s3_key(src, target);

        assert!(result.is_err());
        assert_eq!(
            result.unwrap_err().to_string(),
            "Target path should have at least two components: alias and bucket"
        );
    }
}

pub async fn put(opt: &PutOptions) -> Result<(), Box<dyn std::error::Error>> {
    if opt.src.is_empty() {
        return Err("Path is empty".into());
    }
    let (alias, bucket, key) = generate_s3_key(&opt.src, &opt.target)?;
    let cli = crate::s3::client::get_s3client_from_alias(&alias)?;
    println!("keys is {}", key);

    let mut file = File::open(&opt.src).await?;
    let file_size = file.metadata().await?.len();
    let progress_bar = ProgressBar::new(file_size);
    let progress_style = ProgressStyle::default_bar()
    .template("{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {bytes}/{total_bytes} ({bytes_per_sec}, {eta})")
    .expect("Failed to create progress style") // Unwrap or handle the error here
    .progress_chars("#>-");
    progress_bar.set_style(progress_style);

    // Initiate a multipart upload
    let create_resp = cli
        .create_multipart_upload()
        .bucket(bucket.clone())
        .key(key.clone())
        .send()
        .await?;

    let upload_id = create_resp
        .upload_id()
        .expect("Failed to get upload ID")
        .to_string();

    let mut offset = 0;
    let mut part_number = 1;
    let completed_parts = Arc::new(Mutex::new(Vec::new()));

    // Read and upload chunks concurrently
    let mut handles = Vec::new();
    let s3_client = Arc::new(cli);

    loop {
        let mut buffer = vec![0; CHUNK_SIZE];
        let mut total_bytes_read = 0;

        while total_bytes_read < CHUNK_SIZE {
            let bytes_read = file.read(&mut buffer[total_bytes_read..]).await?;
            if bytes_read == 0 {
                break; // End of file reached
            }
            total_bytes_read += bytes_read;
        }

        if total_bytes_read == 0 {
            break;
        }

        println!("size is {}", total_bytes_read);

        let cli = Arc::clone(&s3_client);
        let upload_id = upload_id.clone();
        let bucket = bucket.clone();
        let key = key.clone();
        let progress_bar = progress_bar.clone();
        let completed_parts = Arc::clone(&completed_parts);
        let buffer = buffer[..total_bytes_read].to_vec();
        let current_part_number = part_number;

        // Spawn a new task for each chunk upload
        let handle = tokio::spawn(async move {
            let part_resp = cli
                .upload_part()
                .bucket(bucket)
                .key(key)
                .upload_id(upload_id)
                .part_number(current_part_number)
                .body(ByteStream::from(buffer))
                .send()
                .await
                .expect("Failed to upload part");

            let e_tag = part_resp.e_tag().expect("Failed to get ETag").to_string();
            progress_bar.inc(total_bytes_read as u64);

            let mut completed_parts_lock = completed_parts.lock().await;
            completed_parts_lock.push(
                CompletedPart::builder()
                    .set_part_number(Some(current_part_number))
                    .set_e_tag(Some(e_tag))
                    .build(),
            );
        });

        handles.push(handle);
        part_number += 1;
    }

    // Wait for all uploads to finish
    for handle in handles {
        handle.await?;
    }

    // Complete the multipart upload
    let mut completed_parts = completed_parts.lock().await;
    //completed_parts.sort_by(compare);
    completed_parts.sort_by(|a, b| a.part_number.cmp(&b.part_number));
    let completed_upload = CompletedMultipartUpload::builder()
        .set_parts(Some(completed_parts.clone()))
        .build();

    s3_client
        .complete_multipart_upload()
        .bucket(bucket)
        .key(key)
        .upload_id(upload_id)
        .multipart_upload(completed_upload)
        .send()
        .await?;

    progress_bar.finish_with_message("Upload complete");
    Ok(())
}
