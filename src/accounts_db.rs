use csv::{ReaderBuilder, Trim, Writer};
use std::{
    collections::BTreeMap,
    io::{Read, Write},
};

use crate::account::Account;
use crate::transaction::Transaction;

// Use a BTreeMap as we want records to be sorted and in general it's more efficient.

/// A Client account database.
#[derive(Debug, Default)]
pub struct AccountsDb(BTreeMap<u16, Account>);

impl AccountsDb {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn process_transactions<R>(&mut self, read: R)
    where
        R: Read,
    {
        let mut reader = ReaderBuilder::new()
            .trim(Trim::All)
            .flexible(true)
            .from_reader(read);

        for result in reader.deserialize() {
            let tx: Transaction = match result {
                Ok(tx) => tx,
                Err(e) => {
                    eprintln!("Failed to parse a transaction record: {}", e);

                    // Just ignore the transaction then.
                    continue;
                }
            };

            // If account doesn't already exist, create one.
            let account = self
                .0
                .entry(tx.client)
                .or_insert_with_key(|id| Account::new(*id));

            if let Err(e) = account.execute_transaction(tx) {
                eprintln!("Failed to process a transaction record: {}", e);
            }
        }
    }

    pub(crate) fn write<W>(&self, write: W) -> Result<(), csv::Error>
    where
        W: Write,
    {
        let mut writer = Writer::from_writer(write);

        for account in self.0.values() {
            writer.serialize(&account)?;
        }

        Ok(())
    }
}

// FIXME: This should work and we shouldn't need the write method above but for some reason we end
//        up with headers duplicated n times, where n = number of accounts. Perhaps another `csv`
//        bug/limitation? Need further investigation.
/*
impl Serialize for AccountsDb {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(self.0.len()))?;
        for account in self.0.values() {
            seq.serialize_element(account)?;
        }

        seq.end()
    }
}
*/
