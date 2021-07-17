use std::{cell::RefCell, convert::TryInto, rc::Rc};

use bobascript::{
  value::{NativeFunction, Value},
  vm::{RuntimeError, VM},
};

fn get(params: &[Value]) -> Result<Value, RuntimeError> {
  if params.len() != 1 {
    Err(RuntimeError::IncorrectParameterCount(
      1,
      params.len().try_into().unwrap(),
    ))
  } else {
    Ok(Value::get_unit())
  }
}

fn set(params: &[Value]) -> Result<Value, RuntimeError> {
  if params.len() != 2 {
    Err(RuntimeError::IncorrectParameterCount(
      2,
      params.len().try_into().unwrap(),
    ))
  } else {
    Ok(Value::get_unit())
  }
}

pub fn add_functions(vm: &mut VM) {
  vm.define_native(
    "get".to_owned(),
    Rc::new(RefCell::new(NativeFunction { function: get })),
  );
  vm.define_native(
    "set".to_owned(),
    Rc::new(RefCell::new(NativeFunction { function: set })),
  );
}
