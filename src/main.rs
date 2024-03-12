use crate::pass_through::PassThrough;
use clap::{Parser, Subcommand};
use controller_state::ControllerState;
use log::info;
use tokio::io::{stdout, AsyncRead};
use tokio::net::TcpListener;

mod controller_state;
mod pass_through;

// Controller controller :)
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Option<Commands>,
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
    env_logger::init();

    let args = Args::parse();

    let mut reader: Box<dyn AsyncRead + Unpin> = match args.command {
        Some(Commands::TCP { ip4, port }) => {
            let listener = TcpListener::bind(format!("{ip4}:{port}")).await?;
            info!("TCP listening on {ip4}:{port}");

            let (stream, address) = listener.accept().await?;
            info!("{address} connected");

            Some(Box::new(stream) as Box<dyn AsyncRead + Unpin>)
        }
        _ => None,
    }
    .ok_or("Failed to create input stream")?;

    info!("Starting pass through");

    loop {
        ControllerState::pass_through(&mut reader, &mut stdout()).await?;
    }
}
