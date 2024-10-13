use crate::cmd::ls::ls;

#[derive(clap::Args, Debug)]
pub struct LsOptions {
    #[arg(help = "section")]
    pub path: String,

    #[arg(long, help = "list all object versions no later than specified date")]
    pub rewind: Option<String>,

    #[arg(long, help = "list all versions")]
    pub versions: bool,

    #[arg(short, long, help = "list recursively")]
    pub recursive: bool,

    #[arg(short = 'I', long, help = "list incomplete uploads")]
    pub incomplete: bool,

    #[arg(
        long,
        help = "display summary information (number of objects, total size)"
    )]
    pub summarize: bool,

    #[arg(long, short = 's', help = "filter to specified storage class")]
    pub storage_class: Option<String>,

    #[arg(long, help = "list files inside zip archive (MinIO servers only)")]
    pub zip: bool,

    #[arg(long, short = 'C', help = "path to configuration folder")]
    pub config_dir: Option<String>,

    #[arg(short, long, help = "disable progress bar display")]
    pub quiet: bool,

    #[arg(
        long = "disable-pager",
        short = 'p',
        help = "disable mc internal pager and print to raw stdout"
    )]
    pub disable_pager: bool,

    #[arg(long, help = "disable color theme")]
    pub no_color: bool,

    #[arg(long, help = "enable JSON lines formatted output")]
    pub json: bool,

    #[arg(long, help = "enable debug output")]
    pub debug: bool,

    #[arg(
        long,
        help = "resolves HOST[:PORT] to an IP address. Example: minio.local:9000=10.10.75.1"
    )]
    pub resolve: Option<String>,

    #[arg(long, help = "disable SSL certificate verification")]
    pub insecure: bool,

    #[arg(
        long,
        help = "limits uploads to a maximum rate (e.g., KiB/s, MiB/s, GiB/s)"
    )]
    pub limit_upload: Option<String>,

    #[arg(
        long,
        help = "limits downloads to a maximum rate (e.g., KiB/s, MiB/s, GiB/s)"
    )]
    pub limit_download: Option<String>,
}

// 处理 run 命令的逻辑
pub async fn handle_ls_commands(opt: &LsOptions) {
    println!("handle list command");
    let _ = ls(opt).await;
}
