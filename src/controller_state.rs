use crate::pass_through::PassThrough;
use devices::controller_state::{ArduinoControllerState, ControllerState};
use log::{debug, error, info};
use std::intrinsics::transmute;
use std::io::{Error, ErrorKind};
use std::mem;
use tokio::io;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

const CONTROLLER_STATE_SIZE: usize = mem::size_of::<ControllerState>();

impl PassThrough for ControllerState {
    async fn pass_through<TWriter, TReader>(
        reader: &mut TReader,
        writers: Vec<&mut TWriter>,
    ) -> io::Result<()>
    where
        TReader: AsyncRead + Unpin,
        TWriter: AsyncWrite + Unpin,
    {
        let mut buffer = [0u8; CONTROLLER_STATE_SIZE];
        let bytes = reader.read(&mut buffer).await?;

        if bytes != CONTROLLER_STATE_SIZE {
            let message = format!(
                "Read {bytes} bytes from reader, but it should be {CONTROLLER_STATE_SIZE} bytes."
            );

            error!("{}", message);
            return Err(Error::new(ErrorKind::BrokenPipe, message));
        }

        debug!("buffer = {:?}", buffer);

        unsafe {
            let state: ControllerState = transmute(buffer);

            info!("{:?}", state);

            let arduino_state = ArduinoControllerState::from_controller_state(&state);

            info!("{:?}", arduino_state);

            let bytes: &[u8] = core::slice::from_raw_parts(
                (&arduino_state as *const ArduinoControllerState) as *const u8,
                mem::size_of::<ArduinoControllerState>(),
            );

            for writer in writers {
                writer.write(bytes).await?;
            }
        }

        Ok(())
    }
}
