use crate::pass_through::PassThrough;
use clap::{Parser, Subcommand};
use devices::controller_state::ControllerState;
use devices::xbox_file::XboxFile;
use log::{debug, info};
use tokio::fs::{File, OpenOptions};
use tokio::io::{stdin, stdout, AsyncRead, AsyncWrite};
use tokio::net::TcpListener;
use tokio_serial::SerialStream;

mod controller_state;
mod pass_through;

// Controller controller :)
#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,

    #[arg(long, global = true)]
    /// Write output to file
    file: Option<String>,

    #[arg(long, global = true)]
    /// Write output to serial
    serial: Option<String>,

    #[arg(long, global = true, default_value = "9600")]
    /// Serial baud rate
    baud_rate: Option<u32>,

    #[arg(long, global = true)]
    /// Write to stdout
    stdout: bool,
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

    let mut output_streams = setup_output_streams(&args).await?;

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
        let binding = output_streams.iter_mut().map(|stream| stream).collect();
        ControllerState::pass_through(&mut reader, binding).await?;
    }
}

async fn setup_output_streams(
    args: &Args,
) -> Result<Vec<Box<dyn AsyncWrite + Unpin>>, Box<dyn std::error::Error>> {
    let mut output_streams = vec![];

    // Open output file
    if let Some(path) = &args.file {
        let file = Box::new(
            OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(path.clone())
                .await?,
        ) as Box<dyn AsyncWrite + Unpin>;

        output_streams.push(file);

        info!("Opened file {:?} as output", path);
    };

    // Open serial
    if let Some(serial) = &args.serial {
        let available_serials = tokio_serial::available_ports()?;
        debug!("Available serial ports: {:?}", available_serials);

        if let Ok(_) = available_serials
            .iter()
            .find(|&info| *info.port_name == *serial)
            .ok_or("Could not open the serial")
        {
            let baud_rate = args.baud_rate.unwrap();
            let builder = tokio_serial::new(serial.clone(), baud_rate);
            let stream = SerialStream::open(&builder)?;
            output_streams.push(Box::new(stream) as Box<dyn AsyncWrite + Unpin>);

            info!("Opened serial port {:?} with {baud_rate} baud rate", serial);
        }
    }

    if output_streams.is_empty() || args.stdout {
        output_streams.push(Box::new(stdout()) as Box<dyn AsyncWrite + Unpin>);

        info!("Using stdout as output");
    }

    Ok(output_streams)
}
