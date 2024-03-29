use std::io::{stdout, Error};

use transactions::{
    core::{ledger::Ledger, transaction::Transaction},
    dump_to_csv, read_from_file,
};

fn main() -> Result<(), Error> {
    let csv_path = std::env::args()
        .nth(1)
        .expect("Expected a CSV filename, run with `cargo run -- transactions.csv`");

    let records = read_from_file(csv_path)?.filter_map(|x| Transaction::try_from(x).ok());
    let mut ledger = Ledger::default();
    for tx in records {
        match ledger.handle_transaction(&tx) {
            Ok(_) => {
                // do something with the tx(e.g add it to persistent log)
            }
            Err(_) => {
                // error handling goes here
            }
        }
    }
    dump_to_csv(ledger.entries(), stdout()).map_err(|err| err.into())
}
