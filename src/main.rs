use std::{
    env::args,
    fs::File,
    io::{self, Read, Write},
    process::exit,
};

mod account;
mod accounts_db;
mod f64;
mod transaction;

use accounts_db::AccountsDb;

fn main() {
    let mut args = args();

    if args.len() < 2 {
        // unwrap() is fine here since we always have the 0th argument.
        eprintln!("Usage: {} CSV_FILE_WITH_TRANSACTIONS", args.next().unwrap());

        exit(-1);
    }
    // unwrap is fine since we established just above that we've at least 2 args.
    let filename = args.nth(1).unwrap();
    let file = match File::open(&filename) {
        Ok(f) => f,
        Err(e) => {
            eprintln!("Error reading from file: {}", e);

            exit(-2);
        }
    };

    run(file, io::stdout());
}

fn run<R, W>(read: R, write: W)
where
    R: Read,
    W: Write,
{
    let mut accounts = AccountsDb::new();
    accounts.process_transactions(read);
    if let Err(e) = accounts.write(write) {
        eprintln!("Failed to serialize accounts: {}", e);
    }
}

#[cfg(test)]
mod integration {
    use super::*;

    macro_rules! process_transactions {
        ($input: literal, $expected_output: literal) => {
            let mut output: Vec<u8> = vec![];
            run($input.as_bytes(), &mut output);
            let output = String::from_utf8(output).unwrap();
            assert_eq!(output, $expected_output);
        };
    }

    #[test]
    fn simple() {
        // All valid transactions expept for 1 overdraw request (should be ignored).
        process_transactions!(
            "type,       client, tx, amount
             deposit,    1,      1,  1.00003
             deposit,    2,      2,  2.1111111
             deposit,    1,      3,  2.0
             withdrawal, 1,      4,  1.5
             withdrawal, 2,      5,  3.0\
            ",
            "client,available,held,total,locked\n\
             1,1.5,0.0,1.5,false\n\
             2,2.1111,0.0,2.1111,false\n\
            "
        );
    }

    #[test]
    fn chargeback() {
        // Transactions with 1 valid chargeback and 1 invalid one (should be ignored).
        process_transactions!(
            "type,       client, tx, amount
             deposit,    77,     1,  1.5
             deposit,    80,     2,  2.0
             withdrawal, 77,     3,  1.0
             dispute,    77,     3,
             chargeback, 77,     3,
             chargeback, 80,     5,\
            ",
            "client,available,held,total,locked\n\
            77,-0.5,0.0,-0.5,true\n\
            80,2.0,0.0,2.0,false\n\
            "
        );
    }

    #[test]
    fn resolve() {
        // Transactions with 1 valid dispute resolution and 1 invalid one (should be ignored).
        process_transactions!(
            "type,       client, tx, amount
             deposit,    77,     1,  1.5
             deposit,    80,     2,  2.0
             withdrawal, 77,     3,  1.0
             withdrawal, 80,     4,  1.0
             dispute,    77,     3,
             resolve,    77,     3,
             resolve,    80,     4,
            ",
            "client,available,held,total,locked\n\
            77,0.5,0.0,0.5,false\n\
            80,1.0,0.0,1.0,false\n\
            "
        );
    }
}
