use crate::{Amount, ClientId, CsvAccount};

#[derive(Debug, Clone, Copy)]
pub enum AccountStatus {
    Active,
    Frozen,
}

impl Default for AccountStatus {
    fn default() -> Self {
        Self::Active
    }
}

#[derive(Default, Debug, PartialEq, Eq)]
pub struct Balance {
    pub(crate) amount: Amount,
}

impl Balance {
    fn transfer(&mut self, target: &mut Balance, amount: Amount) {
        self.amount -= amount;
        target.amount += amount;
    }
}

#[derive(Default, Debug)]
pub struct Account {
    pub available: Balance,
    pub held: Balance,
    pub status: AccountStatus,
}

impl Account {
    pub fn total(&self) -> Amount {
        self.available.amount + self.held.amount
    }
    pub fn deposit(&mut self, source: &mut Balance, amount: Amount) {
        source.transfer(&mut self.available, amount)
    }
    pub fn withdraw(&mut self, target: &mut Balance, amount: Amount) {
        // if a client tries to withdraw more than available we just ignore the operation
        if self.available.amount >= amount {
            self.available.transfer(target, amount)
        }
    }
    pub fn hold(&mut self, amount: Amount) {
        self.available.transfer(&mut self.held, amount)
    }

    pub fn resolve(&mut self, amount: Amount) {
        self.held.transfer(&mut self.available, amount)
    }

    pub fn chargeback(&mut self, target: &mut Balance, amount: Amount) {
        self.held.transfer(target, amount);
        self.status = AccountStatus::Frozen;
    }
}

impl From<(ClientId, &Account)> for CsvAccount {
    fn from((id, account): (ClientId, &Account)) -> Self {
        CsvAccount {
            client: id,
            available: account.available.amount,
            held: account.held.amount,
            total: account.available.amount + account.held.amount,
            locked: !matches!(account.status, AccountStatus::Active),
        }
    }
}
