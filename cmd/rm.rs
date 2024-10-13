use aws_sdk_s3::{
    operation::delete_objects,
    types::{Delete, ObjectIdentifier},
};
use clap;

#[derive(clap::Args, Debug)]
pub struct RmOptions {
    #[arg(help = "alias/bucket (e.g., rustfs/bucketxyx)")]
    pub path: String,

    // Flags
    #[arg(long, help = "Remove object(s) and all its versions")]
    pub versions: bool,

    #[arg(short, long, help = "Remove recursively")]
    pub recursive: bool,

    #[arg(long, help = "Allow a recursive remove operation")]
    pub force: bool,

    #[arg(long, help = "Allow site-wide removal of objects")]
    pub dangerous: bool,

    #[arg(
        long,
        help = "Roll back object(s) to current version at specified time"
    )]
    pub rewind: Option<String>,

    #[arg(long = "version-id", help = "Delete a specific version of an object")]
    pub version_id: Option<String>,

    #[arg(short = 'I', long, help = "Remove incomplete uploads")]
    pub incomplete: bool,

    #[arg(long, help = "Perform a fake remove operation")]
    pub dry_run: bool,

    #[arg(long, help = "Read object names from STDIN")]
    pub stdin: bool,

    #[arg(
        long = "older-than",
        help = "Remove objects older than value (e.g., 7d10h31s)"
    )]
    pub older_than: Option<String>,

    #[arg(
        long = "newer-than",
        help = "Remove objects newer than value (e.g., 7d10h31s)"
    )]
    pub newer_than: Option<String>,

    #[arg(long, help = "Bypass governance")]
    pub bypass: bool,

    #[arg(
        long = "non-current",
        help = "Remove object(s) versions that are non-current"
    )]
    pub non_current: bool,

    #[arg(
        long = "config-dir",
        short = 'C',
        help = "Path to configuration folder"
    )]
    pub config_dir: Option<String>,

    #[arg(short, long, help = "Disable progress bar display")]
    pub quiet: bool,

    #[arg(
        long = "disable-pager",
        help = "Disable internal pager and print to raw stdout"
    )]
    pub disable_pager: bool,

    #[arg(long = "no-color", help = "Disable color theme")]
    pub no_color: bool,

    #[arg(long, help = "Enable JSON lines formatted output")]
    pub json: bool,

    #[arg(long, help = "Enable debug output")]
    pub debug: bool,

    #[arg(long, help = "Resolve HOST[:PORT] to an IP address")]
    pub resolve: Option<String>,

    #[arg(long, help = "Disable SSL certificate verification")]
    pub insecure: bool,

    #[arg(
        long = "limit-upload",
        help = "Limit upload rate (KiB/s, MiB/s, GiB/s)"
    )]
    pub limit_upload: Option<String>,

    #[arg(
        long = "limit-download",
        help = "Limit download rate (KiB/s, MiB/s, GiB/s)"
    )]
    pub limit_download: Option<String>,
}

pub async fn handle_rm_command(opt: &RmOptions) -> Result<(), Box<dyn std::error::Error>> {
    rm(opt).await
}

fn generate_s3_key(path: &str) -> Result<(String, String, String), String> {
    let target_path = std::path::Path::new(path);

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

    let key = s3_key.to_string_lossy().into_owned();

    Ok((alias, bucket, key))
}

pub async fn rm(opt: &RmOptions) -> Result<(), Box<dyn std::error::Error>> {
    let (alias, bucket, key) = generate_s3_key(&opt.path)?;
    let cli = crate::s3::client::get_s3client_from_alias(&alias)?;

    if opt.recursive {
        let mut continuation_token = None;

        loop {
            // List objects with the given prefix
            let resp = cli
                .list_objects_v2()
                .bucket(&bucket)
                .prefix(&key)
                .set_continuation_token(continuation_token.clone())
                .send()
                .await?;

            // Collect object keys to delete
            if !resp.contents().is_empty() {
                let keys: Result<Vec<ObjectIdentifier>, aws_sdk_s3::error::BuildError> = resp
                    .contents
                    .iter()
                    .flat_map(|objects| {
                        objects.iter().filter_map(|object| {
                            object
                                .key
                                .as_ref()
                                .map(|key| ObjectIdentifier::builder().key(key.clone()).build())
                        })
                    })
                    .collect();

                let keys = match keys {
                    Ok(keys) => keys,
                    Err(err) => {
                        eprintln!("Error building ObjectIdentifier: {}", err);
                        return Err(Box::new(err)); // Return an error if building fails
                    }
                };
                // Create the Delete object
                let delete = Delete::builder().set_objects(Some(keys)).build();

                if delete.is_ok() {
                    cli.delete_objects()
                        .bucket(&bucket)
                        .delete(delete.unwrap())
                        .send()
                        .await?;
                }
            }

            // Check if there are more objects to list
            if resp.is_truncated().unwrap() {
                continuation_token = resp.next_continuation_token().map(|s| s.to_string());
            } else {
                break;
            }
        }

        Ok(())
    } else {
        match cli.delete_object().bucket(&bucket).key(&key).send().await {
            Ok(_) => {
                println!(
                    "Object '{}' in bucket '{}' deleted successfully",
                    &key, &bucket
                );
                Ok(())
            }
            Err(err) => {
                println!("Error deleting object: {}", err);
                Err(Box::new(err))
            }
        }
    }
}
