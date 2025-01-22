use std::collections::HashMap;

use fjall::{Config, Keyspace, PartitionHandle};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::error::CliError;
use crate::types::{ClientId, Transaction, TransactionType};

#[derive(Debug, Serialize, Deserialize)]
pub struct Account {
    client: ClientId,
    available: Decimal,
    held: Decimal,
    total: Decimal,
    locked: bool,
}

impl Account {
    pub fn new(id: ClientId) -> Self {
        Self {
            client: id,
            available: dec!(0.0),
            held: dec!(0.0),
            total: dec!(0.0),
            locked: false,
        }
    }

    pub fn deposit(&mut self, amount: Decimal) {
        self.available += amount;
        self.total = self.available + self.held;
    }

    pub fn withdraw(&mut self, amount: Decimal) -> bool {
        if self.available > amount {
            self.available -= amount;
            self.total = self.available + self.held;
            true
        } else {
            false
        }
    }

    pub fn dispute(&mut self, amount: Decimal) {}
    pub fn resolve(&mut self, amount: Decimal) {}
    pub fn freeze(&mut self, amount: Decimal) {}
}

const TX_PART: &str = "transactions";

#[derive(Debug, Serialize, Deserialize)]
enum TransactionRecord {
    Deposit {
        client: ClientId,
        amount: Decimal,
        disputed: bool,
    },
    Withdrawal {
        client: ClientId,
        amount: Decimal,
        succeeded: bool,
        disputed: bool,
    },
}

impl TransactionRecord {
    pub fn deposit(client: ClientId, amount: Decimal) -> Self {
        Self::Deposit {
            client,
            amount,
            disputed: false,
        }
    }

    pub fn withdrawal(client: ClientId, amount: Decimal, succeeded: bool) -> Self {
        Self::Withdrawal {
            client,
            amount,
            succeeded,
            disputed: false,
        }
    }
}
pub struct AccountStates {
    _tx_db: Keyspace,
    transactions: PartitionHandle,
    record: HashMap<ClientId, Account>,
}

impl AccountStates {
    pub fn new() -> Result<Self, CliError> {
        let tx_db = Config::new("./temp_db").temporary(true).open()?;
        let transactions = tx_db.open_partition(TX_PART, Default::default())?;
        Ok(Self {
            _tx_db: tx_db,
            transactions,
            record: Default::default(),
        })
    }

    pub fn submit(&mut self, tx: Transaction) -> Result<(), CliError> {
        // all amounts are now valid
        // we can safely unwrap deposits and withdrawal amounts
        tx.validate()?;

        match tx.tx_type {
            TransactionType::Deposit => {
                let deposit = TransactionRecord::deposit(tx.client, tx.amount.unwrap());
                let encoded = bincode::serialize(&deposit)?;
                self.transactions.insert(&tx.tx.to_be_bytes(), encoded)?;

                let acct = self
                    .record
                    .entry(tx.client)
                    .or_insert(Account::new(tx.client));

                acct.deposit(tx.amount.unwrap())
            }
            TransactionType::Withdrawal => {
                let acct = self
                    .record
                    .entry(tx.client)
                    .or_insert(Account::new(tx.client));

                let successful = acct.withdraw(tx.amount.unwrap());

                let withdrawal =
                    TransactionRecord::withdrawal(tx.client, tx.amount.unwrap(), successful);
                let encoded = bincode::serialize(&withdrawal)?;
                self.transactions.insert(&tx.tx.to_be_bytes(), encoded)?;
            }
            TransactionType::Dispute => {
                let Some(encoded) = self.transactions.get(&tx.tx.to_be_bytes())? else {
                    return Ok(());
                };
            }
            TransactionType::Resolve => {
                let Some(encoded) = self.transactions.get(&tx.tx.to_be_bytes())? else {
                    return Ok(());
                };
            }
            TransactionType::ChargeBack => {
                let Some(encoded) = self.transactions.get(&tx.tx.to_be_bytes())? else {
                    return Ok(());
                };
            }
        }

        Ok(())
    }

    pub fn finish(self) -> Vec<Account> {
        self.record.into_values().collect()
    }
}
