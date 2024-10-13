#[derive(clap::Args, Debug)]
pub struct RbOptions {
    #[arg(help = "alias/bucket   (rustfs/bucketxyx)")]
    pub path: String,
    #[clap(
        long,
        action,
        help = "Force a recursive remove operation on all object versions"
    )]
    pub force: bool,

    /// Allow site-wide removal of objects
    #[clap(long, action, help = "Allow site-wide removal of objects")]
    pub dangerous: bool,

    /// Path to configuration folder (default: "/home/ldy/.mc")
    #[clap(
        long,
        short = 'C',
        default_value = "/home/ldy/.mc",
        help = "Path to configuration folder"
    )]
    pub config_dir: String,

    /// Disable progress bar display
    #[clap(long, short = 'q', action, help = "Disable progress bar display")]
    pub quiet: bool,

    /// Disable mc internal pager and print to raw stdout
    #[clap(
        long = "disable-pager",
        alias = "dp",
        action,
        help = "Disable mc internal pager and print to raw stdout"
    )]
    pub disable_pager: bool,

    /// Disable color theme
    #[clap(long = "no-color", action, help = "Disable color theme")]
    pub no_color: bool,

    /// Enable JSON lines formatted output
    #[clap(long, action, help = "Enable JSON lines formatted output")]
    pub json: bool,

    /// Enable debug output
    #[clap(long, action, help = "Enable debug output")]
    pub debug: bool,

    /// Resolve HOST[:PORT] to an IP address (Example: minio.local:9000=10.10.75.1)
    #[clap(
        long,
        help = "Resolves HOST[:PORT] to an IP address. Example: minio.local:9000=10.10.75.1"
    )]
    pub resolve: Option<String>,

    /// Disable SSL certificate verification
    #[clap(long, action, help = "Disable SSL certificate verification")]
    pub insecure: bool,

    /// Limits uploads to a maximum rate in KiB/s, MiB/s, GiB/s (default: unlimited)
    #[clap(
        long = "limit-upload",
        help = "Limits uploads to a maximum rate in KiB/s, MiB/s, GiB/s. (default: unlimited)"
    )]
    pub limit_upload: Option<String>,

    /// Limits downloads to a maximum rate in KiB/s, MiB/s, GiB/s (default: unlimited)
    #[clap(
        long = "limit-download",
        help = "Limits downloads to a maximum rate in KiB/s, MiB/s, GiB/s. (default: unlimited)"
    )]
    pub limit_download: Option<String>,
}

pub async fn handle_rb_command(opt: &RbOptions) {
    let _ = rb(opt).await;
}

fn split_first_part(input: &str) -> (&str, &str) {
    let mut parts = input.splitn(2, '/');
    let first_part = parts.next().unwrap_or(""); // 获取 "a1"
    let rest_part = parts.next().unwrap_or(""); // 获取 "a2/a3/a4"

    (first_part, rest_part)
}

async fn rb(opt: &RbOptions) -> Result<(), Box<dyn std::error::Error>> {
    if opt.path.is_empty() {
        println!("path is empty");
        return Err("Path is empty".into());
    }
    let (alias, key) = split_first_part(&opt.path);
    let (bucket, _) = split_first_part(key);
    if bucket == "" {
        println!("bucket is null");
        return Err("Bucket not provided".into());
    }

    let cli = crate::s3::client::get_s3client_from_alias(alias)?;

    //s3_client.create_bucket().bucket(bucket);
    match cli.delete_bucket().bucket(bucket).send().await {
        Ok(_) => {
            println!("Bucket '{}' delete successfully", bucket);
            Ok(())
        }
        Err(err) => {
            println!("{}", err);
            Err(Box::new(err))
        }
    }
}
