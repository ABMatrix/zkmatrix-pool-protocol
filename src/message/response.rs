use erased_serde::Serialize as ErasedSerialize;
use json_rpc_types::Id;
use serde::ser::{Serialize, SerializeSeq};
use serde::{Deserialize, Deserializer, Serializer};
use serde_json::Value;
use tokio_util::codec::{Decoder, Encoder};
use crate::message::stratum::StratumMessage;

pub enum ResponseMessage {
    Bool(bool),
    Array(Vec<Box<dyn ErasedSerialize + Send + Sync>>),
    Null,
}

impl ResponseMessage {
    pub fn name(&self) -> &'static str {
        match self {
            ResponseMessage::Bool(_) => "Bool",
            ResponseMessage::Array(_) => "Array",
            ResponseMessage::Null => "Null",
        }
    }
}

impl Serialize for ResponseMessage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
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
                let _ = a.iter().for_each(|v| match v {
                    Value::String(s) => vec.push(Box::new(s.clone())),
                    Value::Number(n) => vec.push(Box::new(n.as_u64())),
                    Value::Null => vec.push(Box::new(None::<String>)),
                    _ => {}
                });
                Ok(ResponseMessage::Array(vec))
            }
            Value::Null => Ok(ResponseMessage::Null),
            _ => Err(serde::de::Error::custom("invalid response params")),
        }
    }
}

#[test]
fn test_response_array_string() {
    use bytes::BytesMut;
    use crate::message::stratum::StratumCodec;
    let response_params: Vec<Box<dyn ErasedSerialize + Send + Sync>> =
        vec![Box::new("SERVER_AGENT1"), Box::new("SERVER_AGENT2")];

    let pa = ResponseMessage::Array(response_params);

    let response = StratumMessage::Response(Id::Num(1), Some(pa), None);
    let mut codec = StratumCodec::default();
    let mut buf1 = BytesMut::new();
    codec.encode(response, &mut buf1).unwrap();
    println!("{:?}", buf1.clone());
    let result = codec.decode(&mut buf1).unwrap().unwrap();
    if let StratumMessage::Response(id, res, _error) = result {
        println!("{:?}", id);
        match res {
            None => {}
            Some(msg) => {
                match msg {
                    ResponseMessage::Array(agent) => {
                        assert_eq!(agent.len(), 2);
                        println!("{}", serde_json::to_string(&agent[0]).unwrap());
                        println!("{}", serde_json::to_string(&agent[1]).unwrap());
                    }
                    _ => {}
                }
            }
        }
    }
}
