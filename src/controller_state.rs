use crate::pass_through::PassThrough;
use std::intrinsics::transmute;
use std::{io, mem};
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt};

#[repr(C)]
#[derive(Debug)]
pub struct ControllerState {
    pub left_thumb: i16,
    pub right_thumb: i16,
}
const CONTROLLER_STATE_SIZE: usize = mem::size_of::<ControllerState>();

impl PassThrough for ControllerState {
    async fn pass_through<TWriter, TReader>(
        reader: &mut TReader,
        writer: &mut TWriter,
    ) -> io::Result<()>
    where
        TReader: AsyncRead + Unpin,
        TWriter: AsyncWrite + Unpin,
    {
        let mut buffer = [0u8; CONTROLLER_STATE_SIZE];
        reader.read(&mut buffer).await?;

        unsafe {
            let state: ControllerState = transmute(buffer);
            writer.write(format!("{:?}\n", state).as_bytes()).await?;
        }

        Ok(())
    }
}
