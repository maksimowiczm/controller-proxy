use tokio::io;
use tokio::io::{AsyncRead, AsyncWrite};

pub trait PassThrough {
    async fn pass_through<TWriter, TReader>(
        reader: &mut TReader,
        writer: &mut TWriter,
    ) -> io::Result<()>
    where
        TReader: AsyncRead + Unpin,
        TWriter: AsyncWrite + Unpin;
}
