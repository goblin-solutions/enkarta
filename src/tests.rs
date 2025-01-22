use super::*;
use csv::{ReaderBuilder, WriterBuilder};
use job_queue::Account;
use std::io::Cursor;
use types::TransactionType;

#[test]
fn deserialize() -> Result<(), CliError> {
    let input = "\
type,client,tx,amount
deposit,1,1,1.0
withdrawal,2,5,3.0
dispute,1,1,
resolve,2,5,
chargeback,1,1,";

    let cursor = Cursor::new(input);
    let mut reader = ReaderBuilder::new().from_reader(cursor);
    let txs: Vec<Transaction> = reader.deserialize().collect::<Result<_, _>>()?;
    assert_eq!(txs.len(), 5);

    assert!(matches!(txs[0].tx_type, TransactionType::Deposit));
    txs[0].validate()?;

    assert!(matches!(txs[1].tx_type, TransactionType::Withdrawal));
    txs[1].validate()?;

    assert!(matches!(txs[2].tx_type, TransactionType::Dispute));
    txs[2].validate()?;

    assert!(matches!(txs[3].tx_type, TransactionType::Resolve));
    txs[3].validate()?;

    assert!(matches!(txs[4].tx_type, TransactionType::ChargeBack));
    txs[4].validate()?;

    Ok(())
}

#[test]
fn serialize() -> Result<(), CliError> {
    let accounts = vec![Account::new(0), Account::new(1)];

    let cursor = Cursor::new(Vec::new());
    let mut writer = WriterBuilder::new().from_writer(cursor);

    for acct in accounts {
        writer.serialize(acct)?;
    }

    writer.flush().map_err(|_| CliError::WriteError)?;

    let cursor = writer.into_inner().unwrap();
    let vec = cursor.into_inner();
    let written = String::from_utf8(vec).unwrap();

    let expected = "\
client,available,held,total,locked
0,0.0,0.0,0.0,false
1,0.0,0.0,0.0,false
";

    assert_eq!(written, expected);
    Ok(())
}
