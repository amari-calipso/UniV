use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{api_field_def_fn, api_field_typedef_fn, api_fn_body, api_layer, api_layers::unil::log, compiler::type_system::UniLType, univm::object::{AnonObject, UniLValue}};

api_fn_body! {
    println(args, [UniLType::Any, UniLType::Any], univ, task) -> (UniLType::Null) {
        args.remove(0);
        log::func(univ, args, task)
    }
}

api_layer! {
    definitions(globals) {
        let mut system = AnonObject::new();
        {
            let mut out = AnonObject::new();
            api_field_def_fn!(out, println);

            system.set(&Rc::from("out"), UniLValue::Object(Rc::new(RefCell::new(out.into()))));
        }

        globals.define(&Rc::from("System"), UniLValue::Object(Rc::new(RefCell::new(system.into()))));
    }

    types(globals) {
        let mut system = HashMap::new();
        {
            let mut out = HashMap::new();
            api_field_typedef_fn!(out, println);

            system.insert(Rc::from("out"), UniLType::Object { fields: Rc::new(RefCell::new(out)) });
        }

        globals.define(&Rc::from("System"), UniLType::Object { fields: Rc::new(RefCell::new(system)) });
    }
}