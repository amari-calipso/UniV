use std::{cell::RefCell, collections::{HashMap, HashSet}, ops::Range, rc::Rc};

use crate::{compiler::type_system::UniLType, univm::object::{AnonObject, AnyCallable, AnyObject, ExecutionInterrupt, UniLValue}, UniV};

pub fn fetch_str_field(object: &AnonObject, name: &str, type_: &str) -> Result<Rc<str>, Rc<str>> {
    if let UniLValue::String(str) = object.get(name).ok_or(format!("{type_} object is missing '{name}' field"))? {
        Ok(str)
    } else {
        Err(format!("{type_} object '{name}' field is not a string").into())
    }
}

#[allow(dead_code)]
pub fn fetch_int_field_strict(object: &AnonObject, name: &str, type_: &str) -> Result<i64, Rc<str>> {
    if let UniLValue::Int(x) = object.get(name).ok_or(format!("{type_} object is missing '{name}' field"))? {
        Ok(x)
    } else {
        Err(format!("{type_} object '{name}' field is not an integer").into())
    }
}

pub fn fetch_int_field_range_strict(object: &AnonObject, range: Range<i64>, name: &str, type_: &str) -> Result<i64, Rc<str>> {
    if let UniLValue::Int(x) = object.get(&name).ok_or(format!("{type_} object is missing '{name}' field"))? {
        if range.contains(&x) {
            Ok(x)
        } else {
            Err(format!(
                "{type_} object '{name}' field is not in range [{}, {})",
                range.start, range.end
            ).into())
        }
    } else {
        Err(format!("{type_} object '{name}' field is not an integer").into())
    }
}

pub fn fetch_int_field(object: &AnonObject, name: &str, type_: &str) -> Result<i64, Rc<str>> {
    if let UniLValue::Int(value) | UniLValue::Value { value, .. } = object.get(name)
        .ok_or(format!("{type_} object is missing '{name}' field"))? 
    {
        Ok(value)
    } else {
        Err(format!("{type_} object '{name}' field is not an integer").into())
    }
}

#[allow(dead_code)]
pub fn fetch_int_field_range(object: &AnonObject, range: Range<i64>, name: &str, type_: &str) -> Result<i64, Rc<str>> {
    if let UniLValue::Int(value) | UniLValue::Value { value, .. } = object.get(&name)
        .ok_or(format!("{type_} object is missing '{name}' field"))? 
    {
        if range.contains(&value) {
            Ok(value)
        } else {
            Err(format!(
                "{type_} object '{name}' field is not in range [{}, {})",
                range.start, range.end
            ).into())
        }
    } else {
        Err(format!("{type_} object '{name}' field is not an integer").into())
    }
}


#[allow(dead_code)]
pub fn fetch_number_field(object: &AnonObject, name: &str, type_: &str) -> Result<UniLValue, Rc<str>> {
    let value = object.get(name).ok_or(format!("{type_} object is missing '{name}' field"))?;
    if matches!(value, UniLValue::Int(_) | UniLValue::Float(_)) {
        Ok(value)
    } else {
        Err(format!("{type_} object '{name}' field is not a number").into())
    }
}

pub fn fetch_optional_object_field(object: &AnonObject, name: &str) -> UniLValue {
    object.get(name).unwrap_or(UniLValue::Null)
}

pub fn fetch_object_field(object: &AnonObject, name: &str, type_: &str) -> Result<Rc<RefCell<AnyObject>>, Rc<str>> {
    if let UniLValue::Object(obj) = object.get(name)
        .ok_or(format!("{type_} object is missing '{name}' field"))? 
    {
        Ok(obj)
    } else {
        Err(format!("{type_} object '{name}' field is not an integer").into())
    }
}

pub fn fetch_optional_bool_field(object: &AnonObject, name: &str, default: bool) -> bool {
    if let Some(x) = object.get(name) {
        x.is_truthy()
    } else {
        default
    }
}

pub fn expect_object<'a>(value: &'a UniLValue, msg: &str, univ: &mut UniV) -> Result<&'a Rc<RefCell<AnyObject>>, ExecutionInterrupt> {
    if let UniLValue::Object(obj) = value {
        Ok(obj)
    } else {
        Err(univ.vm.create_exception(UniLValue::String(format!(
            "Expecting Object as {} but got {}",
            msg, value.stringify_type()
        ).into())))
    }
}

#[allow(dead_code)]
pub fn expect_callable(value: &UniLValue, msg: &str, univ: &mut UniV) -> Result<AnyCallable, ExecutionInterrupt> {
    if let UniLValue::Object(obj) = value {
        if let AnyObject::AnyCallable(callable) = &*obj.borrow() {
            Ok(callable.clone())
        } else {
            Err(univ.vm.create_exception(UniLValue::String(format!(
                "Expecting Callable as {} but got {}",
                msg, value.stringify_type()
            ).into())))
        }
    } else {
        Err(univ.vm.create_exception(UniLValue::String(format!(
            "Expecting Object as {} but got {}",
            msg, value.stringify_type()
        ).into())))
    }
}

pub fn expect_int_strict(value: &UniLValue, msg: &str, univ: &mut UniV) -> Result<i64, ExecutionInterrupt> {
    if let UniLValue::Int(x) = value {
        Ok(*x)
    } else {
        Err(univ.vm.create_exception(UniLValue::String(format!(
            "Expecting Int as {} but got {}",
            msg, value.stringify_type()
        ).into())))
    }
}

pub fn expect_int_range_strict(value: &UniLValue, range: Range<i64>, msg: &str, univ: &mut UniV) -> Result<i64, ExecutionInterrupt> {
    if let UniLValue::Int(x) = value {
        if range.contains(x) {
            Ok(*x)
        } else {
            Err(univ.vm.create_exception(UniLValue::String(format!(
                "{} must be in range [{}, {})",
                msg, range.start, range.end
            ).into())))
        }
    } else {
        Err(univ.vm.create_exception(UniLValue::String(format!(
            "Expecting Int as {} but got {}",
            msg, value.stringify_type()
        ).into())))
    }
}

pub fn expect_int(value: &UniLValue, msg: &str, univ: &mut UniV) -> Result<i64, ExecutionInterrupt> {
    if let UniLValue::Int(value) | UniLValue::Value { value, .. }= value {
        Ok(*value)
    } else {
        Err(univ.vm.create_exception(UniLValue::String(format!(
            "Expecting Int as {} but got {}",
            msg, value.stringify_type()
        ).into())))
    }
}

#[allow(dead_code)]
pub fn expect_int_range(value: &UniLValue, range: Range<i64>, msg: &str, univ: &mut UniV) -> Result<i64, ExecutionInterrupt> {
    if let UniLValue::Int(value) | UniLValue::Value { value, .. }= value {
        if range.contains(value) {
            Ok(*value)
        } else {
            Err(univ.vm.create_exception(UniLValue::String(format!(
                "{} must be in range [{}, {})",
                msg, range.start, range.end
            ).into())))
        }
    } else {
        Err(univ.vm.create_exception(UniLValue::String(format!(
            "Expecting Int as {} but got {}",
            msg, value.stringify_type()
        ).into())))
    }
}

pub fn expect_float(value: &UniLValue, msg: &str, univ: &mut UniV) -> Result<f64, ExecutionInterrupt> {
    if let UniLValue::Float(x) = value {
        Ok(*x)
    } else {
        Err(univ.vm.create_exception(UniLValue::String(format!(
            "Expecting Float as {} but got {}",
            msg, value.stringify_type()
        ).into())))
    }
}

pub fn expect_number(value: &UniLValue, msg: &str, univ: &mut UniV) -> Result<f64, ExecutionInterrupt> {
    Ok(match value {
        UniLValue::Int(value) | UniLValue::Value { value, .. } => *value as f64,
        UniLValue::Float(x) => *x,
        _ => {
            return Err(univ.vm.create_exception(UniLValue::String(format!(
                "Expecting Float or Int as {} but got {}",
                msg, value.stringify_type()
            ).into())))
        }
    })
}

pub fn expect_string(value: &UniLValue, msg: &str, univ: &mut UniV) -> Result<Rc<str>, ExecutionInterrupt> {
    if let UniLValue::String(x) = value {
        Ok(Rc::clone(x))
    } else {
        Err(univ.vm.create_exception(UniLValue::String(format!(
            "Expecting String as {} but got {}",
            msg, value.stringify_type()
        ).into())))
    }
}

pub fn expect_optional_string(value: &UniLValue, msg: &str, univ: &mut UniV) -> Result<Option<Rc<str>>, ExecutionInterrupt> {
    if let UniLValue::String(x) = value {
        Ok(Some(Rc::clone(x)))
    } else if matches!(value, UniLValue::Null) {
        Ok(None)
    } else {
        Err(univ.vm.create_exception(UniLValue::String(format!(
            "Expecting String or null as {} but got {}",
            msg, value.stringify_type()
        ).into())))
    }
}

pub fn object_type() -> UniLType {
    UniLType::Object { fields: Rc::new(RefCell::new(HashMap::new())) }
}

pub fn exclusive_number_type() -> UniLType {
    UniLType::Group(HashSet::from([UniLType::Int, UniLType::Float]))
}

pub fn number_type() -> UniLType {
    UniLType::Group(HashSet::from([UniLType::Int, UniLType::Value, UniLType::Float]))
}

pub fn integer_type() -> UniLType {
    UniLType::Group(HashSet::from([UniLType::Int, UniLType::Value]))
}

pub fn nullable_type(type_: UniLType) -> UniLType {
    type_.make_group(UniLType::Null)
}