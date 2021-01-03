use std::error::Error;

pub trait Processable:Sized {
    fn decode(d:&[u8]) -> Result<Self, Box<dyn Error>>;
    fn encode(&self) -> Result<Vec<u8>, Box<dyn Error>>;
}