use serde::Serialize;
use std::{
    collections::BTreeMap,
    fmt::{self, Display, Formatter},
};

use crate::transaction::{Transaction, TransactionType};

pub(crate) enum Error {
    /// Account is frozen. No transactions allowed.
    Frozen,
    /// Insufficient funds available for the transaction.
    InsufficientFunds,
    /// Missing amount in transaction.
    MissingAmount,
    /// Reference to inexistent transaction (e.g in a dispute transaction).
    MissingTransaction,
    /// Attempted to resolve or chargeback a transaction that is not disputed.
    Undisputed,
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match self {
            Error::Frozen => write!(f, "Account is frozen"),
            Error::InsufficientFunds => {
                write!(f, "Insufficient funds available for the transaction")
            }
            Error::MissingAmount => write!(f, "Amount missing in the transaction record"),
            Error::MissingTransaction => write!(f, "Missing referenced transaction record"),
            Error::Undisputed => {
                write!(
                    f,
                    "Attempt to resolve or chargeback an undisputed transaction"
                )
            }
        }
    }
}

// TODO: serialize (and also maybe deserialize) f64 values with a custom function to only display
// with precision of 4 digits past the decimal.

/// A Client account.
#[derive(Debug, Default, Serialize)]
pub struct Account {
    /// The globally-unique ID for the account.
    #[serde(rename = "client")]
    client_id: u16,
    /// The funds available to the account holder.
    available: f64,
    /// The funds held (in case of disputes) from the account holder.
    held: f64,
    /// The total funds in this account.
    total: f64,
    /// If the account is locked due to a chargeback.
    locked: bool,

    /// All account transactions.
    // Keeping deposit and withdrawal transactions for each account in the memory here and this
    // would be the right thing to do if we want it to be HA and we're not constrained by host
    // resources. Otherwise, we'll want to use a proper DBMS.
    #[serde(skip)]
    transactions: BTreeMap<u32, Transaction>,
}

impl Account {
    pub(crate) fn new(id: u16) -> Self {
        Self {
            client_id: id,
            ..Default::default()
        }
    }

    pub(crate) fn client_id(&self) -> u16 {
        self.client_id
    }

    pub(crate) fn execute_transaction(&mut self, tx: Transaction) -> Result<(), Error> {
        self.ensure_not_frozen()?;

        match tx.r#type {
            TransactionType::Deposit => {
                let amount = tx.amount.ok_or(Error::MissingAmount)?;

                self.deposit(amount);
                self.transactions.insert(tx.tx, tx);
            }
            TransactionType::Withdrawal => {
                let amount = tx.amount.ok_or(Error::MissingAmount)?;

                self.withdraw(amount)?;
                self.transactions.insert(tx.tx, tx);
            }
            // FIXME: Refactor to reduce code duplication between the next 3 cases.
            TransactionType::Dispute => {
                let mut disputed = self
                    .transactions
                    .get_mut(&tx.tx)
                    .ok_or(Error::MissingTransaction)?;
                // FIXME:
                //
                // * Ensure disputed transaction is a withdrawal.
                // * Handle the case of already disputed transaction here.
                disputed.disputed = true;
                let amount = disputed.amount.ok_or(Error::MissingAmount)?;

                self.hold(amount);
            }
            TransactionType::Resolve => {
                let disputed = self
                    .transactions
                    .get_mut(&tx.tx)
                    .ok_or(Error::MissingTransaction)?;
                if !disputed.disputed {
                    return Err(Error::Undisputed);
                }
                let amount = disputed.amount.ok_or(Error::MissingAmount)?;

                self.release(amount);
            }
            TransactionType::Chargeback => {
                let disputed = self
                    .transactions
                    .get_mut(&tx.tx)
                    .ok_or(Error::MissingTransaction)?;
                if !disputed.disputed {
                    return Err(Error::Undisputed);
                }
                let amount = disputed.amount.ok_or(Error::MissingAmount)?;

                self.chargeback(amount);
            }
        }

        Ok(())
    }

    /// Deposite `amount` to the account.
    fn deposit(&mut self, amount: f64) {
        self.available += amount;
        self.total += amount;
    }

    /// Withdraw `amount` from the account.
    fn withdraw(&mut self, amount: f64) -> Result<(), Error> {
        let available = self.available - amount;
        if available < 0.0 {
            return Err(Error::InsufficientFunds);
        }
        self.available = available;
        self.total -= amount;

        Ok(())
    }

    /// Hold `amount` from the account.
    fn hold(&mut self, amount: f64) {
        self.held += amount;
        // No requirements specified for insufficient available funds in this case so assuming we
        // can go negative in this case.
        self.available -= amount;
    }

    /// Relese `amount` from the account.
    fn release(&mut self, amount: f64) {
        self.held -= amount;
        self.available += amount;
    }

    /// Chargeback `amount` from the account.
    fn chargeback(&mut self, amount: f64) {
        self.held -= amount;
        self.total -= amount;
        self.locked = true;
    }

    /// Errors out if the account is frozen.
    fn ensure_not_frozen(&self) -> Result<(), Error> {
        if self.locked {
            return Err(Error::Frozen);
        }

        Ok(())
    }
}

// TODO: Unit tests.
