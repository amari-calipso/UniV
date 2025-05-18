use std::{cell::RefCell, collections::HashMap, rc::Rc};
use crate::{api_layer_fn, compiler::type_system::UniLType, univm::environment::Environment, utils::object::object_type};

api_layer_fn! {
    log(args, [UniLType::Any], _univ, _task) -> (UniLType::Null) {
        crate::log!(TraceLogLevel::LOG_INFO, "{}", args[0].stringify());
        Ok(UniLValue::Null)
    }

    stringify(args, [UniLType::Any], _univ, _task) -> (UniLType::String) {
        Ok(UniLValue::String(args[0].stringify()))
    }

    stringifyType(args, [UniLType::Any], _univ, _task) -> (UniLType::String) {
        Ok(UniLValue::String(args[0].stringify_type()))
    }

    parseInt(args, [UniLType::String], univ, _task) -> (UniLType::Int) {
        let x = expect_string(&args[0], "first argument of 'parseInt'", univ)?;
        Ok(UniLValue::Int(
            x.parse().map_err(|_| univ.vm.create_exception(UniLValue::String(
                format!("Cannot parse an integer from string '{}'", x).into()
            )))?
        ))
    }

    parseFloat(args, [UniLType::String], univ, _task) -> (UniLType::Float) {
        let x = expect_string(&args[0], "first argument of 'parseFloat'", univ)?;
        Ok(UniLValue::Float(
            x.parse().map_err(|_| univ.vm.create_exception(UniLValue::String(
                format!("Cannot parse a number from string '{}'", x).into()
            )))?
        ))
    }

    asAny(args, [UniLType::Any], _univ, _task) -> (UniLType::Any) {
        Ok(args[0].clone())
    }

    int(args, [number_type()], univ, _task) -> (UniLType::Int) {
        match &args[0] {
            UniLValue::Int(value) | UniLValue::Value { value, .. } => Ok(UniLValue::Int(*value)),
            UniLValue::Float(x) => Ok(UniLValue::Int(*x as i64)),
            _ => {
                Err(univ.vm.create_exception(UniLValue::String(format!(
                    "Cannot cast {} to Int{}",
                    args[0].stringify_type(), {
                        if matches!(args[0], UniLValue::String(_)) {
                            ". If you meant to parse an integer from a string, use 'parseInt'"
                        } else {
                            ""
                        }
                    }
                ).into())))
            }
        }
    }

    float(args, [number_type()], univ, _task) -> (UniLType::Int) {
        match &args[0] {
            UniLValue::Int(value) | UniLValue::Value { value, .. } => Ok(UniLValue::Float(*value as f64)),
            UniLValue::Float(x) => Ok(UniLValue::Float(*x)),
            _ => {
                Err(univ.vm.create_exception(UniLValue::String(format!(
                    "Cannot cast {} to Float{}",
                    args[0].stringify_type(), {
                        if matches!(args[0], UniLValue::String(_)) {
                            ". If you meant to parse a float from a string, use 'parseFloat'"
                        } else {
                            ""
                        }
                    }
                ).into())))
            }
        }
    }

    round(args, [number_type()], univ, _task) -> (UniLType::Int) {
        match &args[0] {
            UniLValue::Int(value) | UniLValue::Value { value, .. } => Ok(UniLValue::Int(*value)),
            UniLValue::Float(x) => Ok(UniLValue::Int(x.round() as i64)),
            _ => {
                Err(univ.vm.create_exception(UniLValue::String(format!(
                    "Expecting Float or Int as first argument of 'round' but got {}",
                    args[0].stringify_type()
                ).into())))
            }
        }
    }

    len(args, [UniLType::Group(HashSet::from([UniLType::String, UniLType::List]))], univ, _task) -> (UniLType::Int) {
        match &args[0] {
            UniLValue::String(str) => return Ok(UniLValue::Int(str.len() as i64)),
            UniLValue::Object(obj) => {
                if let AnyObject::List(list) = &*obj.borrow() {
                    return Ok(UniLValue::Int(list.items.len() as i64));
                }
            }
            _ => ()
        }

        Err(univ.vm.create_exception(UniLValue::String(format!("Cannot get length of {}", args[0].stringify_type()).into())))
    }

    randomInt(args, [integer_type(), integer_type()], univ, _task) -> (UniLType::Int) {
        use rand::Rng;
        let a = expect_int(&args[0], "first argument of 'randomInt'", univ)?;
        let b = expect_int(&args[1], "second argument of 'randomInt'", univ)?;
        Ok(UniLValue::Int(univ.rng.random_range(a..=b)))
    }

    randomUniform(_args, [], univ, _task) -> (UniLType::Float) {
        use rand::Rng;
        Ok(UniLValue::Float(univ.rng.random()))
    }

    hasAttribute(args, [UniLType::Any, UniLType::String], univ, _task) -> (UniLType::Int) {
        let name = expect_string(&args[1], "second argument of 'hasAttribute'", univ)?;

        if let UniLValue::Object(obj) = &args[0] {
            if let AnyObject::AnonObject(instance) = &*obj.borrow() {
                if instance.fields.contains_key(&name) {
                    return Ok(UniLValue::Int(1));
                }
            }
        }

        Ok(UniLValue::Int(0))
    }

    setGlobal(args, [UniLType::String, UniLType::Any], univ, _task) -> (UniLType::Null) {
        let name = expect_string(&args[0], "first argument of 'setGlobal'", univ)?;
        
        let res = {
            univ.vm.globals.borrow_mut().set(&name, args[1].clone())
        };
            
        res.map_err(|_| univ.vm.create_exception(UniLValue::String(
            format!("Unknown variable '{}'", name).into()
        )))?;

        Ok(UniLValue::Null)
    }

    contains(args, [UniLType::List, UniLType::Any], univ, _task) -> (UniLType::Int) {
        let obj = expect_object(&args[0], "first argument of 'contains'", univ)?;

        if let AnyObject::List(list) = &*obj.borrow() {
            Ok(UniLValue::Int(list.contains(&args[1]) as i64))
        } else {
            Err(univ.vm.create_exception(UniLValue::String(format!(
                "Expecting list as first argument of 'contains' but got {}",
                args[0].stringify_type()
            ).into())))
        }
    }

    math_pow(args, [number_type(), number_type()], univ, _task) {
        match &args[0] {
            UniLValue::Int(x) | UniLValue::Value { value: x, .. } => {
                match &args[1] {
                    UniLValue::Int(y) | UniLValue::Value { value: y, .. } => {
                        if *y >= 0 {
                            return Ok(UniLValue::Int(x.pow(*y as u32)));
                        } else {
                            return Ok(UniLValue::Float((*x as f64).powi(*y as i32)));
                        }
                    }
                    UniLValue::Float(y) => return Ok(UniLValue::Float((*x as f64).powf(*y))),
                    _ => ()
                }
            }
            UniLValue::Float(x) => {
                match &args[1] {
                    UniLValue::Int(y) | UniLValue::Value { value: y, .. } => {
                        return Ok(UniLValue::Float(x.powi(*y as i32)));
                    }
                    UniLValue::Float(y) => return Ok(UniLValue::Float(x.powf(*y))),
                    _ => ()
                }
            }
            _ => ()
        }

        Err(univ.vm.create_exception(UniLValue::String(format!(
            "'math_pow' cannot be called with arguments of type {} and {}",
            args[0].stringify_type(), args[1].stringify_type()
        ).into())))
    }

    math_ceil(args, [number_type()], univ, _task) {
        match &args[0] {
            UniLValue::Int(_) | UniLValue::Value { .. } => return Ok(args[0].clone()),
            UniLValue::Float(x) => return Ok(UniLValue::Int(x.ceil() as i64)),
            _ => ()
        }

        Err(univ.vm.create_exception(UniLValue::String(format!(
            "'math_ceil' cannot be called with argument of type {}",
            args[0].stringify_type()
        ).into())))
    }

    math_floor(args, [number_type()], univ, _task) {
        match &args[0] {
            UniLValue::Int(_) | UniLValue::Value { .. } => return Ok(args[0].clone()),
            UniLValue::Float(x) => return Ok(UniLValue::Int(x.floor() as i64)),
            _ => ()
        }

        Err(univ.vm.create_exception(UniLValue::String(format!(
            "'math_floor' cannot be called with argument of type {}",
            args[0].stringify_type()
        ).into())))
    }

    math_log2(args, [number_type()], univ, _task) {
        match &args[0] {
            UniLValue::Int(x) | UniLValue::Value { value: x, .. } => return Ok(UniLValue::Int(x.ilog2() as i64)),
            UniLValue::Float(x) => return Ok(UniLValue::Float(x.log2())),
            _ => ()
        }

        Err(univ.vm.create_exception(UniLValue::String(format!(
            "'math_log2' cannot be called with argument of type {}",
            args[0].stringify_type()
        ).into())))
    }

    math_sin(args, [UniLType::Float], univ, _task) -> (UniLType::Float) {
        let x = expect_float(&args[0], "first argument of 'math_sin'", univ)?;
        Ok(UniLValue::Float(x.sin()))
    }

    math_sqrt(args, [number_type()], univ, _task) -> (UniLType::Float) {
        let x = expect_number(&args[0], "first argument of 'math_sqrt'", univ)?;
        Ok(UniLValue::Float(x.sqrt()))
    }

    math_log(args, [number_type(), number_type()], univ, _task) -> (UniLType::Float) {
        let x = expect_number(&args[0], "first argument of 'math_log'", univ)?;
        let b = expect_number(&args[1], "second argument of 'math_log'", univ)?;
        Ok(UniLValue::Float(x.log(b)))
    }

    math_abs(args, [number_type()], univ, _task) {
        match &args[0] {
            UniLValue::Int(value) | UniLValue::Value { value, .. } => Ok(UniLValue::Int(value.abs())),
            UniLValue::Float(x) => Ok(UniLValue::Float(x.abs())),
            _ => {
                Err(univ.vm.create_exception(UniLValue::String(format!(
                    "Expecting Float or Int as first argument of 'math_abs' but got {}",
                    args[0].stringify_type()
                ).into())))
            }
        }
    }

    swap(args, [UniLType::List, integer_type(), integer_type()], univ, _task) {
        let obj = expect_object(&args[0], "first argument of 'swap'", univ)?;

        if let AnyObject::List(list) = &mut *obj.borrow_mut() {
            let a = expect_int(&args[1], "second argument of 'swap'", univ)?;
            let b = expect_int(&args[2], "third argument of 'swap'", univ)?;

            let right;
            with_timer!(univ, {
                let tmp = list.get_with_exception(a, univ)?;
                right = list.get_with_exception(b, univ)?;
                list.set(a, right.clone()).unwrap();
                list.set(b, tmp).unwrap();
            });

            univ.swaps += 1;
            univ.writes += 2;

            let aux = univ.get_optional_aux_id(obj.as_ptr() as *const AnyObject);
            univ.highlights.push(HighlightInfo::from_idx_and_aux_write(list.convert_index(a), aux));
            univ.highlights.push(HighlightInfo::from_idx_and_aux_write(list.convert_index(b), aux));

            Ok(right)
        } else {
            Err(univ.vm.create_exception(UniLValue::String(format!(
                "Expecting list as first argument of 'swap' but got {}",
                args[0].stringify_type()
            ).into())))
        }
    }

    Range_next(args, [object_type()], univ, _task) -> (UniLType::Int) {
        let obj = expect_object(&args[0], "first argument of 'Range_next'", univ)?;

        if let AnyObject::AnonObject(instance) = &mut *obj.borrow_mut() {
            let current = fetch_int_field(&instance, "current", "Range")
                .map_err(|e| univ.vm.create_exception(UniLValue::String(e)))?;
            let step = fetch_int_field(&instance, "step", "Range")
                .map_err(|e| univ.vm.create_exception(UniLValue::String(e)))?;
            let end = fetch_int_field(&instance, "end", "Range")
                .map_err(|e| univ.vm.create_exception(UniLValue::String(e)))?;

            if step > 0 {
                if current >= end {
                    return Ok(UniLValue::Null);
                }
            } else {
                if current <= end {
                    return Ok(UniLValue::Null);
                }
            }

            instance.set(&Rc::from("current"), UniLValue::Int(current + step));
            Ok(UniLValue::Int(current))
        } else {
            Err(univ.vm.create_exception(UniLValue::String(format!(
                "Expecting anonymous object as argument of 'Range_next' but got {}",
                args[0].stringify_type()
            ).into())))
        }
    }

    Range(args, [integer_type(), integer_type()], univ, task) -> (super::range_type()) {
        args.push(UniLValue::Int(1));
        super::RangeWithStep::func(univ, args, task)
    }

    RangeWithStep(args, [integer_type(), integer_type(), integer_type()], univ, _task) -> (super::range_type()) {
        let rd = args.pop().unwrap();
        let nd = args.pop().unwrap();
        let st = args.pop().unwrap();

        expect_int(&st, "first argument of 'RangeWithStep'", univ)?;
        expect_int(&nd, "second argument of 'RangeWithStep'", univ)?;
        expect_int(&rd, "third argument of 'RangeWithStep'", univ)?;

        let mut obj = AnonObject::new();
        obj.set(&Rc::from("start"), st.clone());
        obj.set(&Rc::from("end"), nd);
        obj.set(&Rc::from("step"), rd);
        obj.set(&Rc::from("current"), st);
        obj.set(&Rc::from("next"), univ.vm.get_global("Range_next")?);

        Ok(UniLValue::Object(Rc::new(RefCell::new(obj.into()))))
    }

    List_push(args, [UniLType::List, UniLType::Any], univ, _task) -> (UniLType::Null) {
        let obj = expect_object(&args[0], "first argument of 'List_push'", univ)?;

        if let AnyObject::List(list) = &mut *obj.borrow_mut() {
            let idx = list.items.len();

            with_timer!(univ, list.items.push(args[1].clone()));
            univ.writes += 1;

            let aux = univ.get_optional_aux_id(obj.as_ptr() as *const AnyObject);
            univ.highlights.push(HighlightInfo::from_idx_and_aux_write(idx, aux));

            Ok(UniLValue::Null)
        } else {
            Err(univ.vm.create_exception(UniLValue::String(format!(
                "Expecting list as first argument of 'List_push' but got {}",
                args[0].stringify_type()
            ).into())))
        }
    }

    List_clear(args, [UniLType::List], univ, _task) -> (UniLType::Null) {
        let obj = expect_object(&args[0], "first argument of 'List_clear'", univ)?;

        if let AnyObject::List(list) = &mut *obj.borrow_mut() {
            univ.writes += list.items.len() as u64;
            with_timer!(univ, list.items.clear());
            Ok(UniLValue::Null)
        } else {
            Err(univ.vm.create_exception(UniLValue::String(format!(
                "Expecting list as first argument of 'List_clear' but got {}",
                args[0].stringify_type()
            ).into())))
        }
    }

    ListIterator_next(args, [list_iterator_type()], univ, _task) {
        let obj = expect_object(&args[0], "first argument of 'ListIterator_next'", univ)?;

        if let AnyObject::AnonObject(instance) = &mut *obj.borrow_mut() {
            let current = fetch_int_field(&instance, "curr", "ListIterator")
                .map_err(|e| univ.vm.create_exception(UniLValue::String(e)))?;
            let list = fetch_object_field(&instance, "list", "ListIterator")
                .map_err(|e| univ.vm.create_exception(UniLValue::String(e)))?;

            if let AnyObject::List(list) = &*list.borrow() {
                if let Some(item) = list.get(current) {
                    instance.set(&Rc::from("curr"), UniLValue::Int(current + 1));
                    Ok(item)
                } else {
                    Ok(UniLValue::Null)
                }
            } else {
                return Err(univ.vm.create_exception(UniLValue::String(format!(
                    "Expecting list as 'list' field of 'ListIterator' but got {}",
                    list.borrow().stringify_type()
                ).into())));
            }
        } else {
            Err(univ.vm.create_exception(UniLValue::String(format!(
                "Expecting anonymous object as argument of 'ListIterator_next' but got {}",
                args[0].stringify_type()
            ).into())))
        }
    }

    ListIterator(args, [UniLType::List], univ, _task) -> (list_iterator_type()) {
        let obj = expect_object(&args[0], "first argument of 'ListIterator'", univ)?;

        if !matches!(&*obj.borrow_mut(), AnyObject::List(_)) {
            return Err(univ.vm.create_exception(UniLValue::String(format!(
                "Expecting list as first argument of 'ListIterator' but got {}",
                args[0].stringify_type()
            ).into())));
        }

        let mut output = AnonObject::new();
        output.set(&Rc::from("list"), args[0].clone());
        output.set(&Rc::from("curr"), UniLValue::Int(0));
        output.set(&Rc::from("next"), univ.vm.get_global("ListIterator_next")?);
        Ok(UniLValue::Object(Rc::new(RefCell::new(output.into()))))
    }

    __builtin_isList(args, [UniLType::Any], _univ, _task) -> (UniLType::Int) {
        if let UniLValue::Object(obj) = &args[0] {
            Ok(UniLValue::Int(matches!(&*obj.borrow(), AnyObject::List(_)) as i64))
        } else {
            Ok(UniLValue::Int(0))
        }
    }

    Thread(args, [UniLType::Any, UniLType::List], univ, _task) -> (UniLType::Int) {
        let obj0 = expect_object(&args[0], "first argument of 'Thread'", univ)?;
        let obj1 = expect_object(&args[1], "second argument of 'Thread'", univ)?;

        let id;
        if let AnyObject::AnyCallable(AnyCallable::Function(function)) = &*obj0.borrow() {
            if let AnyObject::List(arguments) = &*obj1.borrow() {
                if function.arity() as usize != arguments.items.len() {
                    return Err(univ.vm.create_exception(UniLValue::String(format!(
                        "Expecting {} arguments but got {}", 
                        function.arity(), arguments.items.len()
                    ).into())));
                }

                let mut env = Environment::with_enclosing(Rc::clone(&univ.vm.globals));
                for i in 0 .. function.params.len() {
                    env.define(&function.params[i], arguments.items[i].clone());
                }

                let task = Task::new(
                    function.address, 
                    &Rc::new(RefCell::new(env))
                );

                id = univ.vm.schedule(task);
                Ok(UniLValue::Int(id as i64))
            } else {
                Err(univ.vm.create_exception(UniLValue::String(format!(
                    "Expecting list as second argument of 'Thread' but got {}",
                    args[1].stringify_type()
                ).into())))
            }
        } else {
            if matches!(&*obj0.borrow(), AnyObject::AnyCallable(_)) {
                Err(univ.vm.create_exception(UniLValue::String(
                    Rc::from("Expecting non-native callable as first argument of 'Thread' but got a native one")
                )))
            } else {
                Err(univ.vm.create_exception(UniLValue::String(format!(
                    "Expecting callable as first argument of 'Thread' but got {}",
                    args[0].stringify_type()
                ).into())))
            }
        }
    }

    Thread_start(args, [integer_type()], univ, _task) -> (UniLType::Null) {
        let thread_id = expect_int_range(&args[0], 0..i64::MAX, "first argument of 'Thread_start'", univ)?;
        univ.vm.start_task(thread_id as usize);
        Ok(UniLValue::Null)
    }

    Thread_getOutput(args, [integer_type()], univ, _task) {
        let thread_id = expect_int_range(&args[0], 0..i64::MAX, "first argument of 'Thread_getOutput'", univ)? as usize;
        if let Some(result) = univ.vm.return_values.get(&thread_id) {
            Ok(result.clone())
        } else {
            Err(univ.vm.create_exception(UniLValue::String(format!(
                "Thread '{}' has not finished running, cannot get output",
                thread_id
            ).into())))
        }
    }

    Thread_isRunning(args, [integer_type()], univ, _task) -> (UniLType::Int) {
        let thread_id = expect_int_range(&args[0], 0..i64::MAX, "first argument of 'Thread_isRunning'", univ)? as usize;
        Ok(UniLValue::Int(!univ.vm.return_values.contains_key(&thread_id) as i64))
    }
}

pub fn range_type() -> UniLType {
    UniLType::Object { 
        fields: {
            let mut fields = HashMap::new();
            fields.insert(Rc::from("start"),   UniLType::Int);
            fields.insert(Rc::from("end"),     UniLType::Int);
            fields.insert(Rc::from("step"),    UniLType::Int);
            fields.insert(Rc::from("current"), UniLType::Int);
            fields.insert(
                Rc::from("next"), 
                UniLType::Callable { 
                    args: vec![object_type()], 
                    return_type: Box::new(UniLType::Int) 
                }
            );

            Rc::new(RefCell::new(fields))
        }
    }
}

pub fn list_iterator_type() -> UniLType {
    UniLType::Object { 
        fields: {
            let mut fields = HashMap::new();
            fields.insert(Rc::from("list"), UniLType::List);
            fields.insert(Rc::from("curr"), UniLType::Int);
            fields.insert(
                Rc::from("next"), 
                UniLType::Callable { 
                    args: vec![object_type()], 
                    return_type: Box::new(UniLType::Any) 
                }
            );

            Rc::new(RefCell::new(fields))
        }
    }
}