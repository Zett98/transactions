use thiserror::Error;

use crate::{Amount, ClientId, CsvTransaction, CsvTransactionKind, TxId};

pub enum NormalTransaction {
    Deposit { amount: Amount },
    Withdraw { amount: Amount },
}
pub enum SettlementTransaction {
    Dispute,
    Resolve,
    Chargeback,
}

pub enum Transaction {
    Normal {
        client_id: ClientId,
        tx_id: TxId,
        kind: NormalTransaction,
    },
    SettlementTransaction {
        client_id: ClientId,
        tx_id: TxId,
        kind: SettlementTransaction,
    },
}

#[derive(Error, Debug)]
#[error("Invalid amount in transaction body")]
pub struct TryFromCsvTxError;

impl TryFrom<CsvTransaction> for Transaction {
    type Error = TryFromCsvTxError;

    fn try_from(value: CsvTransaction) -> Result<Self, Self::Error> {
        let translated = match value.kind {
            CsvTransactionKind::Deposit => Self::Normal {
                client_id: value.client_id,
                tx_id: value.tx_id,
                kind: NormalTransaction::Deposit {
                    amount: value
                        .amount
                        .filter(|x| *x > Amount::ZERO)
                        .ok_or(TryFromCsvTxError)?,
                },
            },
            CsvTransactionKind::Withdraw => Self::Normal {
                client_id: value.client_id,
                tx_id: value.tx_id,
                kind: NormalTransaction::Withdraw {
                    amount: value
                        .amount
                        .filter(|x| *x > Amount::ZERO)
                        .ok_or(TryFromCsvTxError)?,
                },
            },
            CsvTransactionKind::Dispute => Self::SettlementTransaction {
                client_id: value.client_id,
                tx_id: value.tx_id,
                kind: SettlementTransaction::Dispute,
            },
            CsvTransactionKind::Resolve => Self::SettlementTransaction {
                client_id: value.client_id,
                tx_id: value.tx_id,
                kind: SettlementTransaction::Resolve,
            },
            CsvTransactionKind::Chargeback => Self::SettlementTransaction {
                client_id: value.client_id,
                tx_id: value.tx_id,
                kind: SettlementTransaction::Chargeback,
            },
        };
        Ok(translated)
    }
}
