use std::{cell::RefCell, rc::Rc};

use crate::{api_field_def_fn, api_fn_body, api_fn_obj, api_layer, api_layers::osv::delay, univm::object::{AnonObject, UniLValue}};
use crate::api_layers::osv::{clearMark, markArray};

api_fn_body! {
    compareValues(args, [UniLType::Any, UniLType::Any, UniLType::Any], univ, task) -> (UniLType::Int) {
        args.remove(0);
        univ.vm.delegate_call = true;
        expect_callable(
            &univ.vm.get_global("compareValues")?, 
            "Expecting 'compareValues' to be a callable", 
            univ
        )?.call(univ, args, task)
    }
}

api_fn_body! {
    compareIndexValue(args, [UniLType::Any, UniLType::List, integer_type(), UniLType::Any, number_type(), UniLType::Int], univ, task) -> (UniLType::Int) {
        // TODO: consider custom delay
        args.remove(0);
        args.pop();
        args.pop();
        univ.vm.delegate_call = true;
        expect_callable(
            &univ.vm.get_global("compareIndexValue")?, 
            "Expecting 'compareIndexValue' to be a callable", 
            univ
        )?.call(univ, args, task)
    }
}

api_fn_body! {
    write(args, [UniLType::Any, UniLType::List, integer_type(), UniLType::Any, number_type(), UniLType::Int, UniLType::Int], univ, task) -> (UniLType::Int) {
        // TODO: consider custom delay and whether to show or not
        args.remove(0);
        args.pop();
        args.pop();
        args.pop();
        univ.vm.delegate_call = true;
        expect_callable(
            &univ.vm.get_global("writeToIndex")?, 
            "Expecting 'writeToIndex' to be a callable", 
            univ
        )?.call(univ, args, task)
    }
}

api_layer! {
    definitions(globals) {
        let mut highlights = AnonObject::new();
        api_field_def_fn!(highlights, markArray);
        api_field_def_fn!(highlights, clearMark);
        
        globals.define(&Rc::from("Highlights"), UniLValue::Object(Rc::new(RefCell::new(highlights.into()))));

        let mut delays = AnonObject::new();
        delays.set(&Rc::from("sleep"), api_fn_obj!(delay));

        globals.define(&Rc::from("Delays"), UniLValue::Object(Rc::new(RefCell::new(delays.into()))));

        let mut reads = AnonObject::new();
        api_field_def_fn!(reads, compareValues);
        api_field_def_fn!(reads, compareIndexValue);

        globals.define(&Rc::from("Reads"), UniLValue::Object(Rc::new(RefCell::new(reads.into()))));
        
        let mut writes = AnonObject::new();
        api_field_def_fn!(writes, write);

        globals.define(&Rc::from("Writes"), UniLValue::Object(Rc::new(RefCell::new(writes.into()))));
    }
}