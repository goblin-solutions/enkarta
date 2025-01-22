use thiserror::Error;

#[derive(Error, Debug)]
pub enum CliError {
    #[error("Usage: kraken_demo <file_path>")]
    NoFileProvided,
    #[error("File '{0}' does not exist")]
    FileNotFound(String),
    #[error("Could not read '{0}'")]
    ReadError(String),
    #[error("Could not write csv")]
    WriteError,
    #[error("CSV processing error: {0}")]
    CsvError(#[from] csv::Error),
    #[error("Invalid Row: Missing amount in push or pull")]
    NullWire,
    #[error("Invalid Row: count included in dispute row")]
    SomeDispute,
    #[error("Negative wire amount.")]
    NegativeWire,
    #[error("Precision greater than four decimals.")]
    BigPrecision,
    #[error("Database error: {0}")]
    Db(#[from] fjall::Error),
    #[error("Cbor error: {0}")]
    Bincode(#[from] serde_cbor::Error),
}
