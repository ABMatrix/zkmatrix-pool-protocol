use serde::ser::{Serialize, SerializeSeq};
use serde::{Deserialize, Deserializer, Serializer};
use serde_json::Value;
use erased_serde::Serialize as ErasedSerialize;

pub enum ResponseMessage {
    Bool(bool),
    Array(Vec<Box<dyn ErasedSerialize + Send + Sync>>),
    Null,
}

impl Serialize for ResponseMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: Serializer {
        match self {
            ResponseMessage::Bool(ok) => serializer.serialize_bool(*ok),
            ResponseMessage::Array(v) => {
                let mut seq = serializer.serialize_seq(Some(v.len()))?;
                for item in v {
                    seq.serialize_element(item)?;
                }
                seq.end()
            }
            ResponseMessage::Null => serializer.serialize_none(),
        }
    }
}

impl<'de> Deserialize<'de> for ResponseMessage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
        where
            D: Deserializer<'de>,
    {
        let value = Value::deserialize(deserializer)?;
        match value {
            Value::Bool(b) => Ok(ResponseMessage::Bool(b)),
            Value::Array(a) => {
                let mut vec: Vec<Box<dyn ErasedSerialize + Send + Sync>> = Vec::new();
                let _ = a.iter().map(|v| match v {
                    Value::String(s) => vec.push(Box::new(s.clone())),
                    Value::Number(n) => vec.push(Box::new(n.as_u64())),
                    _ => {}
                });
                Ok(ResponseMessage::Array(vec))
            }
            Value::Null => Ok(ResponseMessage::Null),
            _ => Err(serde::de::Error::custom("invalid response params")),
        }
    }
}

