use std::{error, ffi::NulError, fmt::Display, string::FromUtf8Error};

use crate::types::{allocator::IreeAllocator, status::IreeStatus};

/// Represents an error returned by IREE.
/// IREE functions return a status code, which is a `u32` value. The IreeError struct assumes the status code is an error code.
#[derive(Debug)]
pub struct IreeError {
    kind: IreeErrorKind,
}

#[derive(Debug)]
pub enum IreeErrorKind {
    Status(IreeStatus, String), // For when the function that returned the status code allocated a string for the error message
    UnallocatedStatus(IreeStatus), // For when the function that returned the status code did not allocate a string for the error message (e.g. when it doesn't have an allocator)
    Other(Box<dyn error::Error>),  // For external errors
    Unknown(String),
}

impl error::Error for IreeError {}

impl From<String> for IreeError {
    fn from(s: String) -> Self {
        Self {
            kind: IreeErrorKind::Unknown(s),
        }
    }
}

impl From<FromUtf8Error> for IreeError {
    fn from(e: FromUtf8Error) -> Self {
        Self {
            kind: IreeErrorKind::Other(Box::new(e)),
        }
    }
}
impl From<NulError> for IreeError {
    fn from(e: NulError) -> Self {
        Self {
            kind: IreeErrorKind::Other(Box::new(e)),
        }
    }
}

impl IreeError {
    pub fn new(kind: IreeErrorKind) -> Self {
        Self { kind }
    }
    pub fn from_status(status: IreeStatus, allocator: &IreeAllocator) -> Self {
        Self {
            kind: IreeErrorKind::Status(status, status.to_string(allocator).unwrap()),
        }
    }
}

impl Display for IreeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match &self.kind {
            IreeErrorKind::Status(status, msg) => write!(f, "IREE status: {:?} {}", status, msg),
            IreeErrorKind::UnallocatedStatus(status) => write!(f, "IREE unallocated status: {:?} (try allocating the error message string using an allocator!)", status),
            IreeErrorKind::Unknown(msg) => write!(f, "IREE unknown error: {}", msg),
            IreeErrorKind::Other(err) => write!(f, "IREE other error: {}", err),
        }
    }
}
