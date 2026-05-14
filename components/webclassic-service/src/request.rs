use std::error::Error;
use std::io::{Error as IoError, ErrorKind as IoErrorKind, Read};

use snafu::{ResultExt, Snafu};

use crate::Interrupt;

pub trait Request: Sized {
    type Error: Error + 'static;

    fn deserialize(data: &[u8]) -> Result<Option<(Self, usize)>, Self::Error>;
}

pub struct RequestReader<S> {
    source: S,
    buffer: Vec<u8>,
}

impl<S> RequestReader<S> {
    pub fn new(source: S) -> Self {
        Self {
            source,
            buffer: Vec::with_capacity(8 * 1024),
        }
    }
}

impl<S> RequestReader<S>
where
    S: Read,
{
    pub fn read_request<R>(
        &mut self,
        interrupt: &Interrupt,
    ) -> Result<R, ReadRequestError<R::Error>>
    where
        R: Request,
    {
        loop {
            if interrupt.is_interrupted() {
                return InterruptedSnafu.fail();
            }

            if let Some((request, consumed)) = R::deserialize(&self.buffer).context(InvalidSnafu)? {
                let consumed = consumed.min(self.buffer.len());
                self.buffer.drain(..consumed);
                return Ok(request);
            }

            let mut tmp = [0u8; 1024];
            let acquired = match self.source.read(&mut tmp) {
                Ok(0) | Err(_) if interrupt.is_interrupted() => {
                    return InterruptedSnafu.fail();
                }
                Err(e) if e.kind() == IoErrorKind::TimedOut => {
                    continue;
                }
                Err(e) => {
                    return Err(ReadRequestError::Read { source: e });
                }
                Ok(n) => n,
            };
            self.buffer.extend_from_slice(&tmp[..acquired]);

            if acquired == 0 {
                if self.buffer.is_empty() {
                    return EndSnafu.fail();
                } else {
                    return TruncatedSnafu.fail();
                }
            }
        }
    }
}

#[derive(Snafu, Debug)]
#[non_exhaustive]
pub enum ReadRequestError<E>
where
    E: Error + 'static,
{
    #[snafu(display("could not read request data"))]
    Read { source: IoError },
    #[snafu(display("could not deserialize request from invalid data"))]
    Invalid { source: E },
    #[snafu(display("could not get more requests after source is closed"))]
    End,
    #[snafu(display("could not deserialize request from truncated data after source is closed"))]
    Truncated,
    #[snafu(display("request reading was interrupted"))]
    Interrupted,
}

#[cfg(test)]
mod tests {
    use std::io::{Cursor, Read, Result as IoResult};

    use crate::InterruptSource;

    use super::*;

    #[derive(Debug, PartialEq)]
    struct LineRequest {
        line: String,
    }

    #[derive(Debug, Snafu)]
    #[snafu(display("invalid line request"))]
    struct LineRequestError;

    impl Request for LineRequest {
        type Error = LineRequestError;

        fn deserialize(data: &[u8]) -> Result<Option<(Self, usize)>, Self::Error> {
            match data.iter().position(|&b| b == b'\n') {
                Some(pos) => {
                    let line = String::from_utf8_lossy(&data[..pos]).to_string();
                    Ok(Some((Self { line }, pos + 1)))
                }
                None => Ok(None),
            }
        }
    }

    struct ChunkedRead {
        chunks: Vec<Vec<u8>>,
        index: usize,
    }

    impl ChunkedRead {
        fn new(chunks: Vec<Vec<u8>>) -> Self {
            Self { chunks, index: 0 }
        }
    }

    impl Read for ChunkedRead {
        fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
            if self.index >= self.chunks.len() {
                return Ok(0);
            }
            let chunk = &self.chunks[self.index];
            let n = chunk.len().min(buf.len());
            buf[..n].copy_from_slice(&chunk[..n]);
            self.index += 1;
            Ok(n)
        }
    }

    #[test]
    fn read_single_line() {
        let interrupt = InterruptSource::new().subscribe();
        let mut reader = RequestReader::new(Cursor::new(b"hello\n"));
        let req = reader.read_request::<LineRequest>(&interrupt).unwrap();
        assert_eq!(req.line, "hello");
    }

    #[test]
    fn read_multiple_requests() {
        let interrupt = InterruptSource::new().subscribe();
        let mut reader = RequestReader::new(Cursor::new(b"foo\nbar\n"));
        assert_eq!(
            reader.read_request::<LineRequest>(&interrupt).unwrap().line,
            "foo"
        );
        assert_eq!(
            reader.read_request::<LineRequest>(&interrupt).unwrap().line,
            "bar"
        );
    }

    #[test]
    fn read_end_on_empty() {
        let interrupt = InterruptSource::new().subscribe();
        let mut reader = RequestReader::new(Cursor::new(b""));
        let result = reader.read_request::<LineRequest>(&interrupt);
        assert!(matches!(result, Err(ReadRequestError::End)));
    }

    #[test]
    fn read_truncated() {
        let interrupt = InterruptSource::new().subscribe();
        let mut reader = RequestReader::new(Cursor::new(b"hel"));
        let result = reader.read_request::<LineRequest>(&interrupt);
        assert!(matches!(result, Err(ReadRequestError::Truncated)));
    }

    #[test]
    fn read_chunked_delivery() {
        let interrupt = InterruptSource::new().subscribe();
        let chunks = vec![b"hel".to_vec(), b"lo\nwo".to_vec(), b"rld\n".to_vec()];
        let mut reader = RequestReader::new(ChunkedRead::new(chunks));
        assert_eq!(
            reader.read_request::<LineRequest>(&interrupt).unwrap().line,
            "hello"
        );
        assert_eq!(
            reader.read_request::<LineRequest>(&interrupt).unwrap().line,
            "world"
        );
    }

    #[test]
    fn read_interrupted() {
        let interrupt = InterruptSource::new();
        interrupt.trigger();

        let mut reader = RequestReader::new(Cursor::new(b"hello\n"));
        let result = reader.read_request::<LineRequest>(&interrupt.subscribe());
        assert!(matches!(result, Err(ReadRequestError::Interrupted)));
    }
}
