#[derive(Debug)]
pub enum Error {
    KeyNotFound,
    UnexpectedError,
    TryFromSliceError(&'static str),
    UTF8Error,
}
