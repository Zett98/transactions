use serde::{Deserialize, Serialize};

use crate::{Amount, ClientId};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct Account {
    pub client: ClientId,
    #[serde(default, with = "super::amount::my_amount")]
    pub available: Amount,
    #[serde(default, with = "super::amount::my_amount")]
    pub held: Amount,
    #[serde(default, with = "super::amount::my_amount")]
    pub total: Amount,
    pub locked: bool,
}

#[cfg(test)]
mod test {
    use crate::Amount;
    use csv::ByteRecord;
    use expect_test::expect;

    use super::Account;

    fn example_data() -> Vec<Account> {
        let account = Account {
            client: 5,
            available: Amount::from_str("100.5500").unwrap(),
            held: Amount::from_str("50.2300").unwrap(),
            total: Amount::from_str("50.2300").unwrap() + Amount::from_str("100.5500").unwrap(),
            locked: false,
        };
        vec![account]
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
                let data: Account = byterec.deserialize(Some(&headers)).unwrap();
                records.push(data)
            }
        }
        assert_eq!(&records, &example_data);
    }

    #[test]
    fn output_sample() {
        let example_data = example_data();

        let expected = expect![[r#"
            client,available,held,total,locked
            5,100.55,50.23,150.78,false
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
