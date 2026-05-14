use std::io::{Error as IoError, Write};

use snafu::{ResultExt, Snafu};

pub trait Response {
    fn serialize(&self) -> Vec<u8>;
}

pub struct ResponseWriter<S> {
    sink: S,
}

impl<S> ResponseWriter<S> {
    pub fn new(sink: S) -> Self {
        Self { sink }
    }
}

impl<S> ResponseWriter<S>
where
    S: Write,
{
    pub fn write_response<R>(&mut self, response: &R) -> Result<(), WriteResponseError>
    where
        R: Response,
    {
        let data = response.serialize();
        self.sink.write_all(&data).context(WriteSnafu)?;
        self.sink.flush().context(WriteSnafu)?;
        Ok(())
    }
}

#[derive(Snafu, Debug)]
#[non_exhaustive]
pub enum WriteResponseError {
    #[snafu(display("could not write response data"))]
    Write { source: IoError },
}
