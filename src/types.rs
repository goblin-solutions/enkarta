use super::CliError;
use rust_decimal::Decimal;
use rust_decimal_macros::dec;
use serde::{self, Deserialize, Serialize};

pub type ClientId = u16;
pub type TxId = u32;

#[derive(Debug, Deserialize, Serialize)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub tx_type: TransactionType,
    pub client: ClientId,
    pub tx: TxId,
    pub amount: Option<Decimal>,
}

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum TransactionType {
    Deposit,
    Withdrawal,
    Dispute,
    Resolve,
    ChargeBack,
}

impl Transaction {
    pub fn validate(&self) -> Result<(), CliError> {
        if matches!(
            self.tx_type,
            TransactionType::Deposit | TransactionType::Withdrawal
        ) && self.amount.is_none()
        {
            return Err(CliError::NullWire);
        }

        if !matches!(
            self.tx_type,
            TransactionType::Deposit | TransactionType::Withdrawal
        ) && self.amount.is_some()
        {
            return Err(CliError::SomeDispute);
        }

        if self.amount.map(|a| a.is_sign_negative()).unwrap_or(false) {
            return Err(CliError::NegativeWire);
        }

        if self.amount.map(|a| a.scale() > 4).unwrap_or(false) {
            return Err(CliError::BigPrecision);
        }

        Ok(())
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub enum TransactionRecord {
    Deposit {
        client: ClientId,
        amount: Decimal,
        succeeded: bool,
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
    pub fn deposit(client: ClientId, amount: Decimal, succeeded: bool) -> Self {
        Self::Deposit {
            client,
            amount,
            succeeded,
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

    pub fn client(&self) -> ClientId {
        match self {
            Self::Deposit { client, .. } => *client,
            Self::Withdrawal { client, .. } => *client,
        }
    }

    pub fn successful_amount(&self) -> Option<Decimal> {
        match self {
            Self::Deposit {
                amount,
                succeeded: true,
                ..
            } => Some(*amount),
            Self::Withdrawal {
                amount,
                succeeded: true,
                ..
            } => Some(*amount),
            _ => None,
        }
    }

    pub fn disputed(&self) -> bool {
        match self {
            Self::Deposit { disputed, .. } => *disputed,
            Self::Withdrawal { disputed, .. } => *disputed,
        }
    }

    pub fn direction(&mut self) -> Decimal {
        match self {
            Self::Deposit { .. } => dec!(1.0),
            Self::Withdrawal { .. } => dec!(-1.0),
        }
    }

    pub fn dispute(&mut self) {
        match self {
            Self::Deposit { disputed, .. } => *disputed = true,
            Self::Withdrawal { disputed, .. } => *disputed = true,
        }
    }

    pub fn resolve_dispute(&mut self) {
        match self {
            Self::Deposit { disputed, .. } => *disputed = false,
            Self::Withdrawal { disputed, .. } => *disputed = false,
        }
    }
}
