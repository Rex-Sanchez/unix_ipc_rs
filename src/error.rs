use std::fmt::Display;

pub type Result<T> = std::result::Result<T,Error>;


#[derive(Debug)]
pub enum Error{
    Bincode(bincode::Error),
    Socket(std::io::Error),
}


impl std::error::Error for Error{}


impl Display for Error{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Bincode(e) => f.write_str(&format!("{:#?}",e)),
            Error::Socket(e) => f.write_str(&format!("{:#?}",e)),
        }
    }
}


impl From<bincode::Error> for Error{
    fn from(value: bincode::Error) -> Self {
        Self::Bincode(value)
    }
}

impl From<std::io::Error> for Error{
    fn from(value: std::io::Error) -> Self {
        Self::Socket(value)
    }
}
