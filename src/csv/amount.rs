pub mod my_amount {
    use serde::{self, de, Deserialize, Deserializer, Serializer};

    use crate::Amount;

    pub fn serialize<S>(amount: &Amount, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&amount.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Amount, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        let dt = Amount::from_str(&s).map_err(|e| de::Error::custom(e.to_string()))?;
        Ok(dt)
    }
}
pub mod my_amount_opt {
    use serde::{self, de, Deserialize, Deserializer, Serializer};

    use crate::Amount;

    pub fn serialize<S>(amount: &Option<Amount>, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_some(&amount.map(|s| s.to_string()))
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Amount>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: Option<&str> = Option::deserialize(deserializer)?;
        s.map(|s| Amount::from_str(s).map_err(|e| de::Error::custom(e.to_string())))
            .transpose()
    }
}
