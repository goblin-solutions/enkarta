use super::CliError;
use rust_decimal::Decimal;
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
