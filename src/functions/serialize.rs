use std::{
  collections::HashMap,
  convert::{TryFrom, TryInto},
};

use bobascript::{value::Value, vm::RuntimeError};
use serde::{Deserialize, Serialize};

use crate::DB;

#[derive(Debug, Clone, Serialize, Deserialize)]
enum SerializedValue {
  Tuple(Vec<SerializedValue>),
  Record(HashMap<String, SerializedValue>),
  Number(f64),
  Boolean(bool),
  String(String),
}
impl TryFrom<Value> for SerializedValue {
  type Error = RuntimeError;

  fn try_from(value: Value) -> Result<Self, Self::Error> {
    match value {
      Value::Tuple(tuple) => Ok(SerializedValue::Tuple(
        tuple
          .iter()
          .map(|v| -> Result<Self, Self::Error> { v.clone().try_into() })
          .collect::<Vec<Result<Self, Self::Error>>>()
          .into_iter()
          .collect::<Result<Vec<Self>, Self::Error>>()?,
      )),
      Value::Record(record) => Ok(SerializedValue::Record(
        record
          .iter()
          .map(|(k, v)| -> Result<(String, Self), Self::Error> {
            v.clone().try_into().map(|v| (k.clone(), v))
          })
          .collect::<Vec<Result<(String, Self), Self::Error>>>()
          .into_iter()
          .collect::<Result<HashMap<String, Self>, Self::Error>>()?,
      )),
      Value::Number(num) => Ok(SerializedValue::Number(num)),
      Value::Boolean(bool) => Ok(SerializedValue::Boolean(bool)),
      Value::String(str) => Ok(SerializedValue::String(str)),
      _ => Err(RuntimeError::OperationNotSupported),
    }
  }
}
impl TryInto<Value> for SerializedValue {
  type Error = RuntimeError;

  fn try_into(self) -> Result<Value, Self::Error> {
    match self {
      Self::Tuple(tuple) => Ok(Value::Tuple(
        tuple
          .iter()
          .map(|v| -> Result<Value, Self::Error> { v.clone().try_into() })
          .collect::<Vec<Result<Value, Self::Error>>>()
          .into_iter()
          .collect::<Result<Box<[Value]>, Self::Error>>()?,
      )),
      Self::Record(record) => Ok(Value::Record(
        record
          .iter()
          .map(|(k, v)| -> Result<(String, Value), Self::Error> {
            v.clone().try_into().map(|v| (k.clone(), v))
          })
          .collect::<Vec<Result<(String, Value), Self::Error>>>()
          .into_iter()
          .collect::<Result<HashMap<String, Value>, Self::Error>>()?,
      )),
      Self::Number(num) => Ok(Value::Number(num)),
      Self::Boolean(bool) => Ok(Value::Boolean(bool)),
      Self::String(str) => Ok(Value::String(str)),
    }
  }
}

pub fn get(params: &[Value]) -> Result<Value, RuntimeError> {
  println!("params: {:?}", params);
  if params.len() != 1 {
    Err(RuntimeError::IncorrectParameterCount(
      1,
      params.len().try_into().unwrap(),
    ))
  } else if let Value::String(key) = params.get(0).unwrap() {
    Ok(
      if let Some(value) = DB.get(key).map_err(|_| RuntimeError::Unknown)? {
        bincode::deserialize::<SerializedValue>(&value)
          .unwrap()
          .try_into()
          .unwrap()
      } else {
        Value::get_unit()
      },
    )
  } else {
    // the key exists, but was not a string
    Err(RuntimeError::OperationNotSupported)
  }
}

pub fn set(params: &[Value]) -> Result<Value, RuntimeError> {
  println!("params: {:?}", params);
  if params.len() != 2 {
    return Err(RuntimeError::IncorrectParameterCount(
      2,
      params.len().try_into().unwrap(),
    ));
  }

  let key = params.get(0).unwrap();
  let value = params.get(1).unwrap();
  if let Value::String(key) = key {
    DB.insert(
      key,
      bincode::serialize(&SerializedValue::try_from(value.clone())?).unwrap(),
    )
    .map_err(|_| RuntimeError::Unknown)?;
    Ok(Value::get_unit())
  } else {
    // the key exists, but was not a string
    Err(RuntimeError::OperationNotSupported)
  }
}
