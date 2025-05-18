use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{api_def_fn, api_fn_body, api_fn_obj, api_fn_types, api_layer, api_typedef_fn, compiler::type_system::UniLType, univm::object::{AnonObject, UniLValue}};

api_fn_body! {
    print(args, [UniLType::Any], univ, task) -> (UniLType::Null) {
        crate::api_layers::unil::log::func(univ, args, task)
    }
}

api_fn_body! {
    Python_range1(args, [integer_type()], univ, task) -> (api_layers::unil::range_type()) {
        args.insert(0, UniLValue::Int(0));
        crate::api_layers::unil::Range::func(univ, args, task)
    }
}

api_fn_body! {
    Python_range2(args, crate::api_layers::unil::Range::args(), univ, task) -> (api_layers::unil::range_type()) {
        crate::api_layers::unil::Range::func(univ, args, task)
    }
}

api_fn_body! {
    Python_range3(args, crate::api_layers::unil::RangeWithStep::args(), univ, task) -> (api_layers::unil::range_type()) {
        crate::api_layers::unil::RangeWithStep::func(univ, args, task)
    }
}

api_fn_body! {
    math_ceil(args, [UniLType::Any, UniLType::Any], univ, task) {
        args.remove(0);
        crate::api_layers::unil::math_ceil::func(univ, args, task)
    }
}

api_fn_body! {
    math_floor(args, [UniLType::Any, UniLType::Any], univ, task) {
        args.remove(0);
        crate::api_layers::unil::math_floor::func(univ, args, task)
    }
}

api_fn_body! {
    math_log2(args, [UniLType::Any, UniLType::Any], univ, task) {
        args.remove(0);
        crate::api_layers::unil::math_log2::func(univ, args, task)
    }
}

api_fn_body! {
    str(args, [UniLType::Any], univ, task) -> (UniLType::String) {
        crate::api_layers::unil::stringify::func(univ, args, task)
    }
}

api_fn_body! {
    bool(args, [UniLType::Any], _univ, _task) -> (UniLType::Int) {
        Ok(UniLValue::Int(args[0].is_truthy() as i64))
    }
}

api_fn_body! {
    Python_int(args, [UniLType::Any], univ, task) -> (UniLType::Int) {
        if matches!(args[0], UniLValue::String(_)) {
            crate::api_layers::unil::parseInt::func(univ, args, task)
        } else {
            crate::api_layers::unil::int::func(univ, args, task)
        }
    }
}

api_fn_body! {
    Python_float(args, [UniLType::Any], univ, task) -> (UniLType::Float) {
        if matches!(args[0], UniLValue::String(_)) {
            crate::api_layers::unil::parseFloat::func(univ, args, task)
        } else {
            crate::api_layers::unil::float::func(univ, args, task)
        }
    }
}

api_fn_body! {
    random_randint(args, [UniLType::Any, integer_type(), integer_type()], univ, task) -> (UniLType::Int) {
        args.remove(0);
        crate::api_layers::unil::randomInt::func(univ, args, task)
    }
}

api_fn_body! {
    random_randrange(args, [UniLType::Any, integer_type(), integer_type()], univ, _task) -> (UniLType::Int) {
        use rand::Rng;
        let a = expect_int(&args[1], "first argument of 'random.randrange'", univ)?;
        let b = expect_int(&args[2], "second argument of 'random.randrange'", univ)?;
        Ok(UniLValue::Int(univ.rng.random_range(a..b)))
    }
}

api_fn_body! {
    random_random(args, [UniLType::Any], univ, task) -> (UniLType::Float) {
        args.remove(0);
        crate::api_layers::unil::randomUniform::func(univ, args, task)
    }
}

api_fn_body! {
    math_sin(args, [UniLType::Any, UniLType::Float], univ, task) -> (UniLType::Float) {
        args.remove(0);
        crate::api_layers::unil::math_sin::func(univ, args, task)
    }
}

api_fn_body! {
    math_sqrt(args, [UniLType::Any, exclusive_number_type()], univ, task) -> (UniLType::Float) {
        args.remove(0);
        crate::api_layers::unil::math_sqrt::func(univ, args, task)
    }
}

api_fn_body! {
    math_log(args, [UniLType::Any, exclusive_number_type(), exclusive_number_type()], univ, task) -> (UniLType::Float) {
        args.remove(0);
        crate::api_layers::unil::math_log::func(univ, args, task)
    }
}

api_fn_body! {
    Python_abs(args, [number_type()], univ, task) {
        crate::api_layers::unil::math_abs::func(univ, args, task)
    }
}

api_layer! {
    definitions(globals) {
        globals.define(&Rc::from("True"), UniLValue::Int(1));
        globals.define(&Rc::from("False"), UniLValue::Int(0));
        globals.define(&Rc::from("None"), UniLValue::Null);
        globals.define(&Rc::from("__name__"), UniLValue::String(Rc::from("__main__")));

        api_def_fn!(globals, print);
        api_def_fn!(globals, str);
        api_def_fn!(globals, Python_range1);
        api_def_fn!(globals, Python_range2);
        api_def_fn!(globals, Python_range3);
        api_def_fn!(globals, bool);
        api_def_fn!(globals, Python_int);
        api_def_fn!(globals, Python_float);
        api_def_fn!(globals, Python_abs);

        let mut math = AnonObject::new();
        math.set(&Rc::from("pi"),    UniLValue::Float(std::f64::consts::PI));
        math.set(&Rc::from("ceil"),  api_fn_obj!(math_ceil));
        math.set(&Rc::from("floor"), api_fn_obj!(math_floor));
        math.set(&Rc::from("log2"),  api_fn_obj!(math_log2));
        math.set(&Rc::from("sin"),   api_fn_obj!(math_sin));
        math.set(&Rc::from("sqrt"),  api_fn_obj!(math_sqrt));
        math.set(&Rc::from("log"),   api_fn_obj!(math_log));

        globals.define(&Rc::from("math"), UniLValue::Object(Rc::new(RefCell::new(math.into()))));

        let mut random = AnonObject::new();
        random.set(&Rc::from("randint"),   api_fn_obj!(random_randint));
        random.set(&Rc::from("randrange"), api_fn_obj!(random_randrange));
        random.set(&Rc::from("random"),    api_fn_obj!(random_random));

        globals.define(&Rc::from("random"), UniLValue::Object(Rc::new(RefCell::new(random.into()))));
    }

    types(globals) {
        api_typedef_fn!(globals, print);
        api_typedef_fn!(globals, str);
        api_typedef_fn!(globals, Python_range1);
        api_typedef_fn!(globals, Python_range2);
        api_typedef_fn!(globals, Python_range3);
        api_typedef_fn!(globals, bool);
        api_typedef_fn!(globals, Python_int);
        api_typedef_fn!(globals, Python_float);
        api_typedef_fn!(globals, Python_abs);
        
        let mut math = HashMap::new();
        math.insert(Rc::from("ceil"),  api_fn_types!(math_ceil));
        math.insert(Rc::from("floor"), api_fn_types!(math_floor));
        math.insert(Rc::from("log2"),  api_fn_types!(math_log2));
        math.insert(Rc::from("sin"),   api_fn_types!(math_sin));
        math.insert(Rc::from("sqrt"),  api_fn_types!(math_sqrt));
        math.insert(Rc::from("log"),   api_fn_types!(math_log));

        globals.define(&Rc::from("math"), UniLType::Object { fields: Rc::new(RefCell::new(math)) });

        let mut random = HashMap::new();
        random.insert(Rc::from("randint"),   api_fn_types!(random_randint));
        random.insert(Rc::from("randrange"), api_fn_types!(random_randrange));
        random.insert(Rc::from("random"),    api_fn_types!(random_random));

        globals.define(&Rc::from("random"), UniLType::Object { fields: Rc::new(RefCell::new(random)) });
    }
}