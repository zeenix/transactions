use csv::{Reader, ReaderBuilder, Trim, Writer};
use std::{collections::BTreeMap, env::args, fs::File, io, path::Path, process::exit};

mod account;
mod transaction;

use account::Account;
use transaction::Transaction;

fn main() {
    let mut args = args();

    if args.len() < 2 {
        // unwrap() is fine here since we always have the 0th argument.
        eprintln!("Usage: {} CSV_FILE_WITH_TRANSACTIONS", args.next().unwrap());

        exit(-1);
    }
    // unwrap is fine since we established just above that we've at least 2 args.
    let filename = args.nth(1).unwrap();
    let mut reader = match reader_from_file(&filename) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("Error reading from file: {}", e);

            exit(-2);
        }
    };

    // Use a BTreeMap as we want records to be sorted and in general it's more efficient.
    // TODO: Move all this to a separate module/type.
    let mut accounts = BTreeMap::<u16, Account>::new();

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
        let account = accounts
            .entry(tx.client)
            .or_insert_with_key(|id| Account::new(*id));

        if let Err(e) = account.execute_transaction(tx) {
            eprintln!("Failed to process a transaction record: {}", e);
        }
    }

    let mut writer = Writer::from_writer(io::stdout());
    for account in accounts.values() {
        if let Err(e) = writer.serialize(account) {
            eprintln!(
                "Failed to serialize account with ID {}: {}",
                account.client_id(),
                e
            );
        }
    }
}

fn reader_from_file(filename: impl AsRef<Path>) -> io::Result<Reader<File>> {
    let file = File::open(&filename)?;

    Ok(ReaderBuilder::new()
        .trim(Trim::All)
        .flexible(true)
        .from_reader(file))
}

// TODO: Add integration tests.
