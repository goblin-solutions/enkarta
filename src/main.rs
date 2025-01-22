mod error;
mod job_queue;
#[cfg(test)]
mod tests;
mod types;

use csv::{Reader, Writer};
use error::CliError;
use job_queue::AccountStates;
use std::env;
use std::path::PathBuf;
use types::Transaction;

fn main() -> Result<(), CliError> {
    let args: Vec<_> = env::args().take(3).collect();
    let [_, file_path] = &args[..] else {
        return Err(CliError::NoFileProvided);
    };

    let path = PathBuf::from(file_path);
    if !path.exists() {
        return Err(CliError::FileNotFound(file_path.to_string()));
    }

    let file =
        std::fs::File::open(&path).map_err(|_| CliError::ReadError(file_path.to_string()))?;
    let mut reader = Reader::from_reader(file);
    let mut queue = AccountStates::new("./tmp_db")?;

    for result in reader.deserialize() {
        let transaction: Transaction = result?;
        queue.submit(transaction)?;
    }

    let accounts = queue.finish();
    let mut writer = Writer::from_writer(std::io::stdout());

    for acct in accounts {
        writer.serialize(acct)?;
    }

    writer.flush().map_err(|_| CliError::WriteError)
}
