use std::collections::HashMap;

use fjall::{Config, Keyspace, PartitionHandle};
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{Deserialize, Serialize};

use crate::error::CliError;
use crate::types::{ClientId, Transaction, TransactionRecord, TransactionType};

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

    pub fn deposit(&mut self, amount: Decimal) -> bool {
        if !self.locked {
            self.available += amount;
            self.total = self.available + self.held;
            true
        } else {
            false
        }
    }

    pub fn withdraw(&mut self, amount: Decimal) -> bool {
        if self.available >= amount && !self.locked {
            self.available -= amount;
            self.total = self.available + self.held;
            true
        } else {
            false
        }
    }

    pub fn dispute(&mut self, amount: Decimal) {
        self.held += amount;
        self.total = self.held + self.available;
    }

    pub fn resolve(&mut self, amount: Decimal) {
        self.held -= amount;
        self.total = self.held + self.available;
    }

    pub fn chargeback(&mut self, amount: Decimal) {
        self.held -= amount;
        self.total = self.held + self.available;
        self.locked = true;
    }
}

const TX_PART: &str = "transactions";

pub struct AccountStates {
    _tx_db: Keyspace,
    transactions: PartitionHandle,
    accounts: HashMap<ClientId, Account>,
}

impl AccountStates {
    pub fn new(db_name: &str) -> Result<Self, CliError> {
        let tx_db = Config::new(db_name).temporary(true).open()?;
        let transactions = tx_db.open_partition(TX_PART, Default::default())?;
        Ok(Self {
            _tx_db: tx_db,
            transactions,
            accounts: Default::default(),
        })
    }

    pub fn submit(&mut self, tx: Transaction) -> Result<(), CliError> {
        tx.validate()?;

        match tx.tx_type {
            TransactionType::Deposit => self.handle_deposit(tx),
            TransactionType::Withdrawal => self.handle_withdrawal(tx),
            TransactionType::Dispute => self.handle_dispute(tx),
            TransactionType::Resolve => self.handle_resolve(tx),
            TransactionType::ChargeBack => self.handle_chargeback(tx),
        }
    }

    pub fn finish(self) -> Vec<Account> {
        self.accounts.into_values().collect()
    }

    fn handle_deposit(&mut self, tx: Transaction) -> Result<(), CliError> {
        let acct = self
            .accounts
            .entry(tx.client)
            .or_insert(Account::new(tx.client));

        let succeeded = acct.deposit(tx.amount.unwrap());

        let deposit = TransactionRecord::deposit(tx.client, tx.amount.unwrap(), succeeded);
        let encoded = bincode::serialize(&deposit)?;
        self.transactions.insert(&tx.tx.to_be_bytes(), encoded)?;
        Ok(())
    }

    fn handle_withdrawal(&mut self, tx: Transaction) -> Result<(), CliError> {
        let acct = self
            .accounts
            .entry(tx.client)
            .or_insert(Account::new(tx.client));

        let successful = acct.withdraw(tx.amount.unwrap());

        let withdrawal = TransactionRecord::withdrawal(tx.client, tx.amount.unwrap(), successful);
        let encoded = bincode::serialize(&withdrawal)?;
        self.transactions.insert(&tx.tx.to_be_bytes(), encoded)?;
        Ok(())
    }

    fn handle_dispute(&mut self, tx: Transaction) -> Result<(), CliError> {
        let Some(encoded) = self.transactions.get(&tx.tx.to_be_bytes())? else {
            return Ok(());
        };

        let mut record: TransactionRecord = bincode::deserialize(&encoded)?;
        let amount = record.successful_amount();

        if record.client() != tx.client || record.disputed() {
            return Ok(());
        }

        let Some(amount) = amount else {
            return Ok(());
        };

        let acct = self
            .accounts
            .entry(tx.client)
            .or_insert(Account::new(tx.client)); // unreachable

        acct.dispute(amount * record.direction());

        record.dispute();
        let encoded = bincode::serialize(&record)?;
        self.transactions.insert(&tx.tx.to_be_bytes(), encoded)?;
        Ok(())
    }

    fn handle_resolve(&mut self, tx: Transaction) -> Result<(), CliError> {
        let Some(encoded) = self.transactions.get(&tx.tx.to_be_bytes())? else {
            return Ok(());
        };

        let mut record: TransactionRecord = bincode::deserialize(&encoded)?;
        let amount = record.successful_amount();

        if record.client() != tx.client || !record.disputed() {
            return Ok(());
        }

        let Some(amount) = amount else {
            return Ok(());
        };

        let acct = self
            .accounts
            .entry(tx.client)
            .or_insert(Account::new(tx.client)); // unreachable

        acct.resolve(amount * record.direction());

        record.resolve_dispute();
        let encoded = bincode::serialize(&record)?;
        self.transactions.insert(&tx.tx.to_be_bytes(), encoded)?;
        Ok(())
    }

    fn handle_chargeback(&mut self, tx: Transaction) -> Result<(), CliError> {
        let Some(encoded) = self.transactions.get(&tx.tx.to_be_bytes())? else {
            return Ok(());
        };

        let record: TransactionRecord = bincode::deserialize(&encoded)?;
        let amount = record.successful_amount();

        if record.client() != tx.client {
            return Ok(());
        }

        let Some(amount) = amount else {
            return Ok(());
        };

        let acct = self
            .accounts
            .entry(tx.client)
            .or_insert(Account::new(tx.client));

        acct.chargeback(amount);
        Ok(())
    }
}
