use std::{cell::RefCell, rc::Rc};

use bobascript::{value::NativeFunction, vm::VM};

pub mod date;
pub mod serialize;

#[macro_export]
macro_rules! arity {
  ($count:literal, $params:expr) => {
    if $params.len() != $count {
      use std::convert::TryInto;
      return Err(RuntimeError::IncorrectParameterCount(
        $count,
        $params.len().try_into().unwrap(),
      ));
    }
  };
}

macro_rules! define_native {
  ($vm:expr, $name:literal, $func:expr) => {
    $vm.define_native(
      $name.to_owned(),
      Rc::new(RefCell::new(NativeFunction { function: $func })),
    );
  };
}

pub fn add_functions(vm: &mut VM) {
  // date functions:
  define_native!(vm, "epoch", date::epoch);
  define_native!(vm, "format_date", date::format_date);
  // serialize functions:
  define_native!(vm, "get", serialize::get);
  define_native!(vm, "set", serialize::set);
}
