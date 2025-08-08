use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{api_field_def_fn, api_field_typedef_fn, api_fn_body, api_fn_obj, api_fn_types, api_layer, api_layers::osv::delay, compiler::type_system::UniLType, univm::object::{AnonObject, UniLValue}};
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
    write(args, [UniLType::Any, UniLType::List, integer_type(), UniLType::Any, number_type(), UniLType::Int, UniLType::Int], univ, task) -> (UniLType::Any) {
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
        // Highlights
        {
            let mut highlights = AnonObject::new();
            api_field_def_fn!(highlights, markArray);
            api_field_def_fn!(highlights, clearMark);
            
            globals.define(&Rc::from("Highlights"), UniLValue::Object(Rc::new(RefCell::new(highlights.into()))));
        }
        
        // Delays
        {
            let mut delays = AnonObject::new();
            delays.set(&Rc::from("sleep"), api_fn_obj!(delay));

            globals.define(&Rc::from("Delays"), UniLValue::Object(Rc::new(RefCell::new(delays.into()))));
        }
        
        // Reads
        {
            let mut reads = AnonObject::new();
            api_field_def_fn!(reads, compareValues);
            api_field_def_fn!(reads, compareIndexValue);

            globals.define(&Rc::from("Reads"), UniLValue::Object(Rc::new(RefCell::new(reads.into()))));
        }
        
        // Writes
        {
            let mut writes = AnonObject::new();
            api_field_def_fn!(writes, write);

            globals.define(&Rc::from("Writes"), UniLValue::Object(Rc::new(RefCell::new(writes.into()))));
        }
    }

    types(globals) {
        // Highlights
        {
            let mut highlights = HashMap::new();
            api_field_typedef_fn!(highlights, markArray);
            api_field_typedef_fn!(highlights, clearMark);
            
            globals.define(&Rc::from("Highlights"), UniLType::Object { fields: Rc::new(RefCell::new(highlights)) });
        }
        
        // Delays
        {
            let mut delays = HashMap::new();
            delays.insert(Rc::from("sleep"), api_fn_types!(delay));

            globals.define(&Rc::from("Delays"), UniLType::Object { fields: Rc::new(RefCell::new(delays)) });
        }
        
        // Reads
        {
            let mut reads = HashMap::new();
            api_field_typedef_fn!(reads, compareValues);
            api_field_typedef_fn!(reads, compareIndexValue);

            globals.define(&Rc::from("Reads"), UniLType::Object { fields: Rc::new(RefCell::new(reads)) });
        }
        
        // Writes
        {
            let mut writes = HashMap::new();
            api_field_typedef_fn!(writes, write);

            globals.define(&Rc::from("Writes"), UniLType::Object { fields: Rc::new(RefCell::new(writes)) });
        }
    }
}