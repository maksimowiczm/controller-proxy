use crate::pass_through::PassThrough;
use clap::{Parser, Subcommand};
use tokio::io::{AsyncRead, stdout};
use tokio::net::TcpListener;
use controller_state::ControllerState;

mod pass_through;
mod controller_state;

// Controller controller :)
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    TCP {
        /// IPv4 socket address
        ip4: String,
        /// Port number
        port: String,
    },
    USB {},
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let mut reader: Box<dyn AsyncRead + Unpin> = match args.command {
        Commands::TCP { ip4, port } => {
            let listener = TcpListener::bind(format!("{ip4}:{port}")).await?;
            let (stream, _) = listener.accept().await?;
            Some(Box::new(stream) as Box<dyn AsyncRead + Unpin>)
        }
        Commands::USB { .. } => None,
    }
    .ok_or("Failed to create input")?;

    loop {
        ControllerState::pass_through(&mut reader, &mut stdout()).await?;
    }
}
