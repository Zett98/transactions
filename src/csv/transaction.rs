use serde::{Deserialize, Serialize};

use crate::common::amount::Amount;

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum TransactionKind {
    Deposit,
    #[serde(rename = "withdrawal")]
    Withdraw,
    Dispute,
    Resolve,
    Chargeback,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Transaction {
    #[serde(rename = "type")]
    pub kind: TransactionKind,
    #[serde(rename = "client")]
    pub client_id: u16,
    #[serde(rename = "tx")]
    pub tx_id: u32,
    #[serde(default, with = "super::amount::my_amount_opt")]
    pub amount: Option<Amount>,
}

#[cfg(test)]
mod test {
    use csv::ByteRecord;
    use expect_test::expect;

    use crate::common::amount::Amount;

    use super::{Transaction, TransactionKind};

    fn example_data() -> Vec<Transaction> {
        let deposit = Transaction {
            kind: TransactionKind::Deposit,
            amount: Some(Amount::from_str("5.5").unwrap()),
            client_id: 6,
            tx_id: 5,
        };
        let withdrawal = Transaction {
            kind: TransactionKind::Withdraw,
            amount: Some(Amount::default()),
            client_id: 6,
            tx_id: 5,
        };
        let dispute = Transaction {
            kind: TransactionKind::Dispute,
            amount: Default::default(),
            client_id: 6,
            tx_id: 5,
        };
        let resolve = Transaction {
            kind: TransactionKind::Resolve,
            amount: Default::default(),
            client_id: 6,
            tx_id: 5,
        };
        let chargeback = Transaction {
            kind: TransactionKind::Chargeback,
            amount: Default::default(),
            client_id: 6,
            tx_id: 5,
        };
        vec![deposit, withdrawal, dispute, resolve, chargeback]
    }

    #[test]
    fn e2e_deser() {
        let example_data = example_data();
        let mut buf = Vec::new();
        // write example records into provided `Writer`
        {
            let mut wtr = csv::Writer::from_writer(&mut buf);
            for i in &example_data {
                wtr.serialize(i).unwrap();
            }
            wtr.flush().unwrap();
        }
        let mut records = vec![];

        // read them from provided `Writer`
        {
            let buf_read = &mut buf.as_slice();
            let mut reader = csv::Reader::from_reader(buf_read);
            let mut byterec = ByteRecord::new();
            let headers = reader.byte_headers().unwrap().clone();
            while reader.read_byte_record(&mut byterec).unwrap() {
                let data: Transaction = byterec.deserialize(Some(&headers)).unwrap();
                records.push(data)
            }
        }
        assert_eq!(&records, &example_data);
    }
    #[test]
    fn output_sample() {
        let example_data = example_data();
        let expected = expect![[r#"
            type,client,tx,amount
            deposit,6,5,5.5
            withdrawal,6,5,0
            dispute,6,5,
            resolve,6,5,
            chargeback,6,5,
        "#]];
        let mut buf = Vec::new();
        // write example records into provided `Writer`
        {
            let mut wtr = csv::Writer::from_writer(&mut buf);
            for i in &example_data {
                wtr.serialize(i).unwrap();
            }
            wtr.flush().unwrap();
        }
        let buf = String::from_utf8(buf).unwrap();
        expected.assert_eq(&buf);
    }
}
