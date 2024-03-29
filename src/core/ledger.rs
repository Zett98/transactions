use std::collections::{btree_map, BTreeMap};
use thiserror::Error;

use super::{
    account::{Account, AccountStatus, Balance},
    transaction::{NormalTransaction, SettlementTransaction, Transaction},
};
use crate::{common::TxId, Amount, ClientId};

pub enum TransactionOutcomeKind {
    Applied,
    Disputed,
    Resolved,
    Chargeback,
}
pub struct DepositOutcome {
    kind: TransactionOutcomeKind,
    amount: Amount,
}

impl DepositOutcome {
    fn applied(amount: Amount) -> Self {
        DepositOutcome {
            kind: TransactionOutcomeKind::Applied,
            amount,
        }
    }
}

#[derive(Debug, Error)]
pub enum Errors {
    #[error("Account with disputed transaction doesn't exist")]
    AccountDoesNotExist,
    #[error("Account with disputed transaction doesn't exist")]
    DisputedNonExistant,
    #[error("Account was frozen during the dispute. reason: Chargeback")]
    AccountFrozen,
}

pub type Result = std::result::Result<(), Errors>;

pub struct Client {
    account: Account,
    deposits: BTreeMap<TxId, DepositOutcome>,
    withdrawals: BTreeMap<TxId, Amount>,
}

impl Client {
    fn wrap_error(&self) -> Result {
        match &self.account.status {
            AccountStatus::Frozen => Err(Errors::AccountFrozen),
            AccountStatus::Active => Ok(()),
        }
    }
    fn handle_dispute(&mut self, tx_id: TxId) -> Result {
        if let btree_map::Entry::Occupied(mut tx) = self.deposits.entry(tx_id) {
            let tx = tx.get_mut();
            if matches!(tx.kind, TransactionOutcomeKind::Applied) {
                tx.kind = TransactionOutcomeKind::Disputed;
                self.account.hold(tx.amount);
            }
        }
        Ok(())
    }
    fn handle_resolve(&mut self, tx_id: TxId) -> Result {
        if let btree_map::Entry::Occupied(mut tx) = self.deposits.entry(tx_id) {
            let tx = tx.get_mut();
            if matches!(tx.kind, TransactionOutcomeKind::Disputed) {
                tx.kind = TransactionOutcomeKind::Resolved;
                self.account.resolve(tx.amount);
            }
        }
        Ok(())
    }
    fn handle_chargeback(&mut self, liabilities: &mut Balance, tx_id: TxId) -> Result {
        let prev_status = self.account.status;
        if let btree_map::Entry::Occupied(mut tx) = self.deposits.entry(tx_id) {
            let tx = tx.get_mut();
            if matches!(tx.kind, TransactionOutcomeKind::Disputed) {
                tx.kind = TransactionOutcomeKind::Chargeback;

                self.account.chargeback(liabilities, tx.amount);
            }
        }
        let new_status = self.account.status;
        match (prev_status, new_status) {
            (AccountStatus::Active, AccountStatus::Frozen) => self.wrap_error(),
            _ => Ok(()),
        }
    }
}

#[derive(Default)]
pub struct Ledger {
    liabilites: Balance,
    accounts: BTreeMap<ClientId, Client>,
}

impl Ledger {
    fn handle_settlement_transaction(
        &mut self,
        client_id: ClientId,
        tx_id: TxId,
        tx: &SettlementTransaction,
    ) -> Result {
        if let Some(client) = self.accounts.get_mut(&client_id) {
            match *tx {
                SettlementTransaction::Dispute => client.handle_dispute(tx_id),
                SettlementTransaction::Resolve => client.handle_resolve(tx_id),
                SettlementTransaction::Chargeback => {
                    client.handle_chargeback(&mut self.liabilites, tx_id)
                }
            }
        } else {
            Ok(())
        }
    }

    fn handle_normal_transaction(
        &mut self,
        client_id: ClientId,
        tx_id: TxId,
        tx: &NormalTransaction,
    ) {
        match *tx {
            NormalTransaction::Deposit { amount } => {
                self.accounts
                    .entry(client_id)
                    .and_modify(|client| {
                        if !matches!(client.account.status, AccountStatus::Active) {
                            return;
                        }
                        client.deposits.entry(tx_id).or_insert_with(|| {
                            client.account.deposit(&mut self.liabilites, amount);
                            DepositOutcome::applied(amount)
                        });
                    })
                    .or_insert_with(|| {
                        let mut account = Account::default();
                        account.deposit(&mut self.liabilites, amount);
                        let mut txs = BTreeMap::new();
                        txs.insert(tx_id, DepositOutcome::applied(amount));
                        Client {
                            account,
                            deposits: txs,
                            withdrawals: Default::default(),
                        }
                    });
            }
            NormalTransaction::Withdraw { amount } => {
                self.accounts.entry(client_id).and_modify(|client| {
                    if !matches!(client.account.status, AccountStatus::Active) {
                        return;
                    }
                    client.withdrawals.entry(tx_id).or_insert_with(|| {
                        client.account.withdraw(&mut self.liabilites, amount);
                        amount
                    });
                });
            }
        }
    }
    pub fn handle_transaction(&mut self, tx: &Transaction) -> Result {
        match tx {
            Transaction::Normal {
                client_id,
                tx_id,
                kind,
            } => {
                self.handle_normal_transaction(*client_id, *tx_id, kind);
                Ok(())
            }
            Transaction::SettlementTransaction {
                client_id,
                tx_id,
                kind,
            } => self.handle_settlement_transaction(*client_id, *tx_id, kind),
        }
    }
    pub fn entries(&self) -> impl Iterator<Item = (ClientId, &'_ Account)> {
        self.accounts
            .iter()
            .map(|(id, client)| (*id, &client.account))
    }
    pub fn get_account(&self, client_id: &ClientId) -> Option<&Account> {
        self.accounts.get(client_id).map(|x| &x.account)
    }
}

#[cfg(test)]
mod test {
    use crate::{
        core::{
            account::{AccountStatus, Balance},
            transaction::{NormalTransaction, SettlementTransaction, Transaction},
        },
        Amount, ClientId, TxId,
    };

    use super::Ledger;

    fn deposit(client_id: ClientId, tx_id: TxId, amount: &str) -> Transaction {
        Transaction::Normal {
            client_id,
            tx_id,
            kind: NormalTransaction::Deposit {
                amount: Amount::from_str(amount).unwrap(),
            },
        }
    }
    fn withdraw(client_id: ClientId, tx_id: TxId, amount: &str) -> Transaction {
        Transaction::Normal {
            client_id,
            tx_id,
            kind: NormalTransaction::Withdraw {
                amount: Amount::from_str(amount).unwrap(),
            },
        }
    }
    fn dispute(client_id: ClientId, tx_id: TxId, kind: SettlementTransaction) -> Transaction {
        Transaction::SettlementTransaction {
            client_id,
            tx_id,
            kind,
        }
    }
    fn execute_tx(ledger: &mut Ledger, txs: &[Transaction]) {
        for tx in txs {
            let _ = ledger.handle_transaction(tx);
        }
    }
    fn ledger_sanity_check(ledger: &mut Ledger) {
        let accounts_total = ledger
            .entries()
            .map(|x| x.1.total())
            .reduce(std::ops::Add::add)
            .unwrap();
        let liabilities = &ledger.liabilites;
        assert_eq!(liabilities.amount + accounts_total, 0)
    }

    fn assert_frozen(client_id: &ClientId, ledger: &Ledger) {
        let status = ledger
            .get_account(client_id)
            .map(|x| x.status)
            .expect("should be present");
        assert!(matches!(status, AccountStatus::Frozen));
    }
    fn assert_active(client_id: &ClientId, ledger: &Ledger) {
        let status = ledger
            .get_account(client_id)
            .map(|x| x.status)
            .expect("should be present");
        assert!(matches!(status, AccountStatus::Active));
    }

    fn assert_available(client_id: &ClientId, ledger: &Ledger, amount: Amount) {
        let available = ledger
            .get_account(client_id)
            .map(|x| &x.available)
            .expect("should be present");
        assert_eq!(*available, Balance { amount });
    }

    #[test]
    fn add_funds_and_withdraw() {
        let mut ledger = Ledger::default();
        let deposit = deposit(1, 1, "15.5301");
        let withdraw = withdraw(1, 2, "5.52");
        let dispute_tx = dispute(1, 1, SettlementTransaction::Dispute);
        let resolve = dispute(1, 1, SettlementTransaction::Resolve);
        let txs = [deposit, withdraw, dispute_tx, resolve];
        execute_tx(&mut ledger, &txs);
        ledger_sanity_check(&mut ledger);
        assert_available(&1, &ledger, Amount::from_str("10.0101").unwrap());
        assert_active(&1, &ledger)
    }

    #[test]
    fn double_entry_resolve() {
        let mut ledger = Ledger::default();
        let deposit = deposit(1, 1, "5.5");
        let withdraw = withdraw(1, 2, "5.5");
        let dispute_tx = dispute(1, 1, SettlementTransaction::Dispute);
        let resolve = dispute(1, 1, SettlementTransaction::Resolve);
        let txs = [deposit, withdraw, dispute_tx, resolve];
        execute_tx(&mut ledger, &txs);
        ledger_sanity_check(&mut ledger);
        assert_available(&1, &ledger, Amount::const_from_int(0));
        assert_active(&1, &ledger)
    }
    #[test]
    fn double_entry_chargeback() {
        let mut ledger = Ledger::default();
        let deposit = deposit(1, 1, "5.5");
        let withdraw = withdraw(1, 2, "3.5");
        let dispute_tx = dispute(1, 1, SettlementTransaction::Dispute);
        let resolve = dispute(1, 1, SettlementTransaction::Chargeback);
        let txs = [deposit, withdraw, dispute_tx, resolve];
        execute_tx(&mut ledger, &txs);
        ledger_sanity_check(&mut ledger);
        assert_available(&1, &ledger, Amount::from_str("-3.5").unwrap());

        assert_frozen(&1, &ledger)
    }

    #[test]
    fn double_entry_chargeback_multi() {
        let mut ledger = Ledger::default();
        let deposit_tx = deposit(1, 1, "5.5");
        let deposit_tx2 = deposit(1, 4, "5.5");
        let withdraw = withdraw(1, 2, "11.");
        let dispute_tx = dispute(1, 1, SettlementTransaction::Dispute);
        let resolve = dispute(1, 1, SettlementTransaction::Chargeback);
        let txs = [deposit_tx, deposit_tx2, withdraw, dispute_tx, resolve];
        execute_tx(&mut ledger, &txs);
        ledger_sanity_check(&mut ledger);
        assert_frozen(&1, &ledger)
    }
}
