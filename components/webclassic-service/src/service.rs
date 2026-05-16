use std::error::Error;
use std::io::{Read, Write};

use snafu::{OptionExt, ResultExt, Snafu};

use crate::interrupt::Interrupt;
use crate::request::{ReadRequestError, Request, RequestReader};
use crate::response::{Response, ResponseWriter, WriteResponseError};

pub trait Service {
    type Request: Request;
    type Response: Response;
    type Error: Error + 'static;

    fn process(
        &self,
        request: Self::Request,
        interrupt: &Interrupt,
    ) -> Result<Option<Self::Response>, Self::Error>;

    fn on_invalid(&self, _error: &<Self::Request as Request>::Error) -> Option<Self::Response> {
        None
    }

    fn run<R, W>(
        &self,
        reader: R,
        writer: W,
        interrupt: &Interrupt,
    ) -> Result<(), RunServiceError<<Self::Request as Request>::Error, Self::Error>>
    where
        R: Read,
        W: Write,
    {
        let mut reader = RequestReader::new(reader);
        let mut writer = ResponseWriter::new(writer);
        loop {
            let request = match reader.read_request(interrupt) {
                Ok(request) => request,
                Err(ReadRequestError::Interrupted) => return InterruptedSnafu.fail(),
                Err(ReadRequestError::End) => return Ok(()),
                Err(e) => {
                    if let ReadRequestError::Invalid { source } = &e
                        && let Some(response) = self.on_invalid(source)
                    {
                        let _ = writer.write_response(&response, interrupt);
                    }
                    return Err(e).context(ReadSnafu);
                }
            };

            if interrupt.is_interrupted() {
                return InterruptedSnafu.fail();
            }

            let response = self
                .process(request, interrupt)
                .context(ProcessSnafu)?
                .context(InterruptedSnafu)?;

            match writer.write_response(&response, interrupt) {
                Ok(()) => (),
                Err(WriteResponseError::Interrupted) => return InterruptedSnafu.fail(),
                err => err.context(WriteSnafu)?,
            }
        }
    }
}

#[derive(Snafu, Debug)]
#[non_exhaustive]
pub enum RunServiceError<RE, PE>
where
    RE: Error + 'static,
    PE: Error + 'static,
{
    #[snafu(display("could not read incoming request"))]
    Read { source: ReadRequestError<RE> },
    #[snafu(display("could not process request"))]
    Process { source: PE },
    #[snafu(display("could not write outgoing response"))]
    Write { source: WriteResponseError },
    #[snafu(display("service was interrupted"))]
    Interrupted,
}
