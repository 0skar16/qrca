use serde::{Serialize, Deserialize};


#[cfg(test)]
mod tests;
pub mod reader;
pub mod writer;
#[derive(Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Entry {
    pub size: u64,
    pub path: String,
}