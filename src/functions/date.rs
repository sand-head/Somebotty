use std::time::{Duration, UNIX_EPOCH};

use bobascript::{value::Value, vm::RuntimeError};
use chrono::{DateTime, Utc};

use crate::arity;

pub fn epoch(params: &[Value]) -> Result<Value, RuntimeError> {
  arity!(0, params);
  let date = Utc::now().timestamp();
  Ok(Value::Number(date as f64))
}

pub fn format_date(params: &[Value]) -> Result<Value, RuntimeError> {
  arity!(2, params);
  let format_str = params.get(0).unwrap();
  let format_str = if let Value::String(str) = format_str {
    str.clone()
  } else {
    return Err(RuntimeError::TypeError {
      expected: "string",
      found: format_str.clone(),
    });
  };
  let timestamp = params.get(1).unwrap();
  let timestamp = if let Value::Number(num) = timestamp {
    *num
  } else {
    return Err(RuntimeError::TypeError {
      expected: "number",
      found: timestamp.clone(),
    });
  };

  let date = UNIX_EPOCH + Duration::from_secs_f64(timestamp);
  let date = DateTime::<Utc>::from(date);
  let formatted_date = date.format(&format_str).to_string();
  Ok(Value::String(formatted_date))
}
