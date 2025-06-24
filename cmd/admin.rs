use clap::Subcommand;

use crate::infocommands;

#[derive(Subcommand)]
pub enum AdminCommands {
    /// Restart or unfreeze a MinIO cluster
    #[command(about = "restart or unfreeze a MinIO cluster")]
    Service,

    /// Update all MinIO servers
    #[command(about = "update all MinIO servers")]
    Update,

    /// Display MinIO server information
    #[command(about = "display MinIO server information")]
    Info,

    /// Manage users
    #[command(about = "manage users")]
    User,

    /// Manage groups
    #[command(about = "manage groups")]
    Group,

    /// Manage policies defined in the MinIO server
    #[command(about = "manage policies defined in the MinIO server")]
    Policy,

    /// Manage MinIO site replication
    #[command(about = "manage MinIO site replication")]
    Replicate,

    /// Manage MinIO server configuration
    #[command(about = "manage MinIO server configuration")]
    Config,

    /// Manage MinIO server pool decommissioning
    #[command(name = "decommission", about = "manage MinIO server pool decommissioning", aliases = &["decom"])]
    Decommission,

    /// Monitor healing for bucket(s) and object(s) on MinIO server
    #[command(about = "monitor healing for bucket(s) and object(s) on MinIO server")]
    Heal,

    /// Manages Prometheus config
    #[command(about = "manages prometheus config")]
    Prometheus,

    /// Perform KMS management operations
    #[command(about = "perform KMS management operations")]
    Kms,

    /// Provide MinIO scanner info
    #[command(about = "provide MinIO scanner info")]
    Scanner,

    /// Provide top-like statistics for MinIO
    #[command(about = "provide top like statistics for MinIO")]
    Top,

    /// Show HTTP call trace for all incoming and internode on MinIO
    #[command(about = "Show HTTP call trace for all incoming and internode on MinIO")]
    Trace,

    /// Manage MinIO cluster metadata
    #[command(about = "manage MinIO cluster metadata")]
    Cluster,

    /// Manage MinIO rebalance
    #[command(about = "Manage MinIO rebalance")]
    Rebalance,

    /// Show MinIO logs
    #[command(about = "show MinIO logs")]
    Logs,
}

pub async fn handle_admin_commands(command: &AdminCommands) {
    match command {
        AdminCommands::Service => {
            println!("Restarting or unfreezing the MinIO cluster...");
        }
        AdminCommands::Update => {
            println!("Updating all MinIO servers...");
        }
        AdminCommands::Info => {
            infocommands::ServerInfo().await;
            println!("Displaying MinIO server information...");
        }
        AdminCommands::User => {
            println!("Managing users...");
        }
        AdminCommands::Group => {
            println!("Managing groups...");
        }
        AdminCommands::Policy => {
            println!("Managing policies defined in the MinIO server...");
        }
        AdminCommands::Replicate => {
            println!("Managing MinIO site replication...");
        }
        AdminCommands::Config => {
            println!("Managing MinIO server configuration...");
        }
        AdminCommands::Decommission => {
            println!("Managing MinIO server pool decommissioning...");
        }
        AdminCommands::Heal => {
            println!("Monitoring healing for buckets and objects on MinIO server...");
        }
        AdminCommands::Prometheus => {
            println!("Managing Prometheus config...");
        }
        AdminCommands::Kms => {
            println!("Performing KMS management operations...");
        }
        AdminCommands::Scanner => {
            println!("Providing MinIO scanner info...");
        }
        AdminCommands::Top => {
            println!("Providing top-like statistics for MinIO...");
        }
        AdminCommands::Trace => {
            println!("Showing HTTP call trace for all incoming and internode on MinIO...");
        }
        AdminCommands::Cluster => {
            println!("Managing MinIO cluster metadata...");
        }
        AdminCommands::Rebalance => {
            println!("Managing MinIO rebalance...");
        }
        AdminCommands::Logs => {
            println!("Showing MinIO logs...");
        }
    }
}
