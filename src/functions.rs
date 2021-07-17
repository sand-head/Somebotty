use std::{cell::RefCell, rc::Rc};

use bobascript::{value::NativeFunction, vm::VM};

pub mod serialize;

pub fn add_functions(vm: &mut VM) {
  vm.define_native(
    "get".to_owned(),
    Rc::new(RefCell::new(NativeFunction {
      function: serialize::get,
    })),
  );
  vm.define_native(
    "set".to_owned(),
    Rc::new(RefCell::new(NativeFunction {
      function: serialize::set,
    })),
  );
}
