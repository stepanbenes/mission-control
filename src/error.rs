#![allow(dead_code)]

use std::error::Error;
use std::fmt;

#[derive(Debug)]
pub struct StringError {
    details: String
}

impl StringError {
    pub fn new(msg: &str) -> Self {
        Self { details: msg.to_string() }
    }
}

impl fmt::Display for StringError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f,"{}",self.details)
    }
}

impl Error for StringError {
    fn description(&self) -> &str {
        &self.details
    }
}