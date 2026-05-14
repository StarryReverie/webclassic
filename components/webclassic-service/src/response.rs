use std::io::{Error as IoError, Write};

use snafu::{ResultExt, Snafu};

use crate::Interrupt;

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
    pub fn write_response<R>(
        &mut self,
        response: &R,
        interrupt: &Interrupt,
    ) -> Result<(), WriteResponseError>
    where
        R: Response,
    {
        if interrupt.is_interrupted() {
            return InterruptedSnafu.fail();
        }

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
    #[snafu(display("response writing was interrupted"))]
    Interrupted,
}

#[cfg(test)]
mod tests {
    use std::io::Cursor;

    use crate::InterruptSource;

    use super::*;

    struct BytesResponse(Vec<u8>);

    impl Response for BytesResponse {
        fn serialize(&self) -> Vec<u8> {
            self.0.clone()
        }
    }

    #[test]
    fn write_response_success() {
        let source = InterruptSource::new();
        let mut writer = ResponseWriter::new(Cursor::new(Vec::new()));
        writer
            .write_response(&BytesResponse(b"hello".to_vec()), &source.subscribe())
            .unwrap();
    }

    #[test]
    fn write_response_interrupted() {
        let source = InterruptSource::new();
        source.trigger();
        let mut writer = ResponseWriter::new(Cursor::new(Vec::new()));
        let result = writer.write_response(&BytesResponse(b"hello".to_vec()), &source.subscribe());
        assert!(matches!(result, Err(WriteResponseError::Interrupted)));
    }
}
