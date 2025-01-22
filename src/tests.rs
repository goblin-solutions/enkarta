use super::*;
use csv::{ReaderBuilder, WriterBuilder};
use job_queue::Account;
use rust_decimal_macros::dec;
use std::io::Cursor;
use types::TransactionType;

macro_rules! deposit {
    ($client:expr, $tx:expr, $amount:expr) => {
        Transaction {
            tx_type: TransactionType::Deposit,
            client: $client,
            tx: $tx,
            amount: Some(dec!($amount)),
        }
    };
}

macro_rules! withdraw {
    ($client:expr, $tx:expr, $amount:expr) => {
        Transaction {
            tx_type: TransactionType::Withdrawal,
            client: $client,
            tx: $tx,
            amount: Some(dec!($amount)),
        }
    };
}

macro_rules! dispute {
    ($client:expr, $tx:expr) => {
        Transaction {
            tx_type: TransactionType::Dispute,
            client: $client,
            tx: $tx,
            amount: None,
        }
    };
}

macro_rules! resolve {
    ($client:expr, $tx:expr) => {
        Transaction {
            tx_type: TransactionType::Resolve,
            client: $client,
            tx: $tx,
            amount: None,
        }
    };
}

macro_rules! chargeback {
    ($client:expr, $tx:expr) => {
        Transaction {
            tx_type: TransactionType::ChargeBack,
            client: $client,
            tx: $tx,
            amount: None,
        }
    };
}

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

/// Q: why expect inside tests?
/// A: because the tracebacks are slightly nicer and
/// I don't want to do the panic catching trick

#[test]
fn basic() {
    let mut states = AccountStates::new("./basic_flow").expect("failed");
    states.submit(deposit!(1, 1, 100.0)).expect("failed");
    states.submit(withdraw!(1, 2, 50.0)).expect("failed");

    let accounts = states.finish();
    let account = accounts.first().expect("failed");

    assert_eq!(account.available(), dec!(50.0));
    assert_eq!(account.total(), dec!(50.0));
}

#[test]
fn dispute_flow() {
    let mut states = AccountStates::new("./dispute_flow").expect("failed");
    states.submit(deposit!(1, 1, 100.0)).expect("failed");
    states.submit(dispute!(1, 1)).expect("failed");
    states.submit(resolve!(1, 1)).expect("failed");

    let accounts = states.finish();
    let account = accounts.first().expect("failed");

    assert_eq!(account.available(), dec!(100.0));
    assert_eq!(account.held(), dec!(0.0));
}

#[test]
fn locked_account_rejects_withdraw() {
    let mut states = AccountStates::new("./locked_flow").expect("failed");
    states.submit(deposit!(1, 1, 100.0)).expect("failed");
    states.submit(dispute!(1, 1)).expect("failed");
    states.submit(chargeback!(1, 1)).expect("failed");
    states.submit(withdraw!(1, 2, 50.0)).expect("failed");

    let accounts = states.finish();
    let account = accounts.first().expect("failed");
    assert_eq!(account.available(), dec!(100.0));
    assert_eq!(account.total(), dec!(100.0));
    assert!(account.locked());
}

#[test]
fn insufficient_funds_withdraw() {
    let mut states = AccountStates::new("./insufficient").expect("failed");
    states.submit(deposit!(1, 1, 100.0)).expect("failed");
    states.submit(withdraw!(1, 2, 150.0)).expect("failed");

    let accounts = states.finish();
    let account = accounts.first().expect("failed");
    assert_eq!(account.available(), dec!(100.0)); // balance unchanged
    assert_eq!(account.total(), dec!(100.0));
}
