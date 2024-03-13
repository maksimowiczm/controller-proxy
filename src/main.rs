use crate::pass_through::PassThrough;
use clap::{Parser, Subcommand};
use devices::controller_state::ControllerState;
use devices::xbox_file::XboxFile;
use log::info;
use tokio::fs::File;
use tokio::io::{stdin, stdout, AsyncRead};
use tokio::net::TcpListener;

mod controller_state;
mod pass_through;

// Controller controller :)
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Use TCP socket
    TCP {
        /// IPv4 socket address
        ip4: String,
        /// Port number
        port: String,
    },
    /// Use XBOX controller USB event file
    USB {
        #[arg(long)]
        /// Path to XBOX controller event file
        event_file: Option<String>,
    },
    /// Use file
    FILE {
        /// File path
        path: String,
    },
    /// Use stdin
    STDIN,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    let args = Args::parse();

    let mut reader: Box<dyn AsyncRead + Unpin> = match args.command {
        Commands::TCP { ip4, port } => {
            let listener = TcpListener::bind(format!("{ip4}:{port}")).await?;
            info!("TCP listening at {ip4}:{port}");

            let (stream, address) = listener.accept().await?;
            info!("{address} connected");

            Some(Box::new(stream) as Box<dyn AsyncRead + Unpin>)
        }
        Commands::FILE { path } => {
            let file = File::open(path.clone()).await?;
            info!("Reading from file {:?}", path);

            Some(Box::new(file) as Box<dyn AsyncRead + Unpin>)
        }
        Commands::USB { event_file } => {
            let (xbox_file, path) = if let Some(path) = event_file {
                XboxFile::from_file(&path)?
            } else {
                XboxFile::from_proc_file()?
            };
            info!("Using xbox event file {:?}", path);

            Some(Box::new(xbox_file) as Box<dyn AsyncRead + Unpin>)
        }
        Commands::STDIN => {
            info!("Using STDIN");

            Some(Box::new(stdin()) as Box<dyn AsyncRead + Unpin>)
        }
    }
    .ok_or("Failed to create input stream")?;

    info!("Starting pass through");

    loop {
        ControllerState::pass_through(&mut reader, &mut stdout()).await?;
    }
}
