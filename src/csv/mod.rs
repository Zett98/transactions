use std::{fs::File, io::Write, path::Path};

use csv::{ByteRecord, Reader};

use crate::CsvTransaction;

pub mod account;
mod amount;
pub mod transaction;

pub fn read_from_file<T: AsRef<Path>>(
    path: T,
) -> Result<impl Iterator<Item = CsvTransaction>, csv::Error> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .has_headers(true)
        .delimiter(b',')
        .flexible(true)
        .double_quote(false)
        .from_path(path)?;
    let mut byterec = ByteRecord::new();
    let headers = reader.byte_headers()?.clone();
    let mut is_finished = false;
    let value_iterator = std::iter::from_fn(move || {
        fn call(
            is_finished: &mut bool,
            reader: &mut Reader<File>,
            headers: &ByteRecord,
            byterec: &mut ByteRecord,
        ) -> Option<CsvTransaction> {
            loop {
                match reader.read_byte_record(byterec) {
                    Ok(true) => {
                        if let Ok(data) = byterec.deserialize::<CsvTransaction>(Some(headers)) {
                            return Some(data);
                        }
                    }
                    Ok(false) => {
                        if let Ok(data) = byterec.deserialize::<CsvTransaction>(Some(headers)) {
                            return Some(data);
                        } else {
                            *is_finished = true;
                            return None;
                        }
                    }
                    Err(_) => {}
                }
            }
        }
        if is_finished {
            return None;
        }
        call(&mut is_finished, &mut reader, &headers, &mut byterec)
    });
    Ok(value_iterator)
}

pub fn dump_to_csv<D, O>(data: impl Iterator<Item = D>, out: O) -> Result<(), csv::Error>
where
    D: Into<account::Account>,
    O: Write,
{
    let mut writer = csv::WriterBuilder::new()
        .delimiter(b',')
        .has_headers(true)
        .flexible(false)
        .double_quote(false)
        .from_writer(out);

    for record in data {
        let for_writing: account::Account = record.into();
        writer.serialize(&for_writing)?
    }
    Ok(())
}
