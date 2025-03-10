use std::io::Cursor;

use bytes::BytesMut;
use tokio::{
    io::{AsyncBufReadExt, AsyncWriteExt, BufReader, BufWriter},
    net::TcpStream,
};

use crate::error::DatabaseResult;

#[derive(Debug)]
pub struct Connection {
    stream: BufWriter<TcpStream>,
    buffer: BytesMut
}

impl Connection {
    pub fn new(stream: TcpStream) -> Self {
        Self { stream: BufWriter::new(stream), buffer: BytesMut::with_capacity(4 * 1024) }
    }   

    pub async fn read(&mut self) {
        let mut cursor = Cursor::new(&self.buffer[..]);

    }

    pub async  fn write_to_stream(&mut self, value: &str) -> DatabaseResult<()> {
        let mut writer = BufWriter::new(&mut self.stream);

        writer.write_all(src)

        Ok(())
    }
}
