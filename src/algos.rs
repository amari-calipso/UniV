use std::{cell::RefCell, rc::Rc};

use crate::{univm::object::{AnonObject, AnyCallable, Callable, NativeCallable, UniLValue}, utils::object::{expect_int_range_strict, fetch_optional_object_field, fetch_str_field}};

pub struct Sort {
    pub callable: AnyCallable,

    pub name:      Rc<str>,
    pub list_name: Rc<str>,
    pub category:  Rc<str>,

    pub killers: UniLValue,
}

impl Sort {
    pub fn new(object: AnonObject, callable: AnyCallable) -> Result<Self, Rc<str>> {
        if callable.arity() != 1 {
            return Err(format!("Sort function has to take 1 parameter (got {})", callable.arity()).into());
        }

        Ok(Sort {
            callable,
            name:      fetch_str_field(&object, "name", "Sort")?,
            list_name: fetch_str_field(&object, "listName", "Sort")?,
            category:  fetch_str_field(&object, "category", "Sort")?,
            killers:   fetch_optional_object_field(&object, "killers")
        })
    }
}

pub struct Shuffle {
    pub callable: AnyCallable,
    pub name: Rc<str>,
}

impl Shuffle {
    pub fn new(object: AnonObject, callable: AnyCallable) -> Result<Self, Rc<str>> {
        if callable.arity() != 1 {
            return Err(format!("Shuffle function has to take 1 parameter (got {})", callable.arity()).into());
        }

        Ok(Shuffle {
            callable,
            name: fetch_str_field(&object, "name", "Shuffle")?,
        })
    }
}

pub struct Distribution {
    pub callable: AnyCallable,
    pub name: Rc<str>,
}

impl Distribution {
    pub fn new(object: AnonObject, callable: AnyCallable) -> Result<Self, Rc<str>> {
        if callable.arity() != 1 {
            return Err(format!("Distribution function has to take 1 parameter (got {})", callable.arity()).into());
        }

        Ok(Distribution {
            callable,
            name: fetch_str_field(&object, "name", "Distribution")?,
        })
    }
}

pub struct PivotSelection {
    pub callable: AnyCallable,
    pub name: Rc<str>,
}

impl PivotSelection {
    pub fn new(object: AnonObject, callable: AnyCallable) -> Result<Self, Rc<str>> {
        if callable.arity() != 3 {
            return Err(format!("Pivot selection function has to take 3 parameters (got {})", callable.arity()).into());
        }

        Ok(PivotSelection {
            callable,
            name: fetch_str_field(&object, "name", "Pivot selection")?,
        })
    }
}

pub struct Rotation {
    pub indexed: AnyCallable,
    pub lengths: AnyCallable,
    pub name: Rc<str>,
}

impl Rotation {
    pub fn new_indexed(object: AnonObject, indexed: AnyCallable) -> Result<Self, Rc<str>> {
        if indexed.arity() != 4 {
            return Err(format!("Rotation function has to take 4 parameters (got {})", indexed.arity()).into());
        }

        let indexed_clone = indexed.clone();

        let lengths = NativeCallable::new(
            Rc::new(
                move |univ, mut args, task| {
                    let a3 = args.pop().unwrap();
                    let a2 = args.pop().unwrap();

                    let a  = expect_int_range_strict(&args[1], 0..i64::MAX, "first argument of 'rotation'", univ)?;
                    let ll = expect_int_range_strict(&a2, 0..i64::MAX, "second argument of 'rotation'", univ)?;
                    let rl = expect_int_range_strict(&a3, 0..i64::MAX, "third argument of 'rotation'", univ)?;

                    let m = a + ll;
                    let b = m + rl;

                    args.push(UniLValue::Int(m));
                    args.push(UniLValue::Int(b));
                    
                    univ.vm.delegate_call = matches!(indexed_clone, AnyCallable::Function(_));
                    indexed_clone.call(univ, args, task)
                }
            ),
            4
        ).into();

        Ok(Rotation {
            indexed,
            lengths,
            name: fetch_str_field(&object, "name", "Rotation")?,
        })
    }

    pub fn new_lengths(object: AnonObject, lengths: AnyCallable) -> Result<Self, Rc<str>> {
        if lengths.arity() != 4 {
            return Err(format!("Rotation function has to take 4 parameters (got {})", lengths.arity()).into());
        }

        let lengths_cloned = lengths.clone();

        let indexed = NativeCallable::new(
            Rc::new(
                move |univ, mut args, task| {
                    let a3 = args.pop().unwrap();
                    let a2 = args.pop().unwrap();

                    let a = expect_int_range_strict(&args[1], 0..i64::MAX, "first argument of 'rotation'", univ)?;
                    let m = expect_int_range_strict(&a2, 0..i64::MAX, "second argument of 'rotation'", univ)?;
                    let b = expect_int_range_strict(&a3, 0..i64::MAX, "third argument of 'rotation'", univ)?;

                    let ll = m - a;
                    let rl = b - m;

                    args.push(UniLValue::Int(ll));
                    args.push(UniLValue::Int(rl));

                    univ.vm.delegate_call = matches!(lengths_cloned, AnyCallable::Function(_));
                    lengths_cloned.call(univ, args, task)
                }
            ),
            4
        ).into();

        Ok(Rotation {
            indexed,
            lengths,
            name: fetch_str_field(&object, "name", "Rotation")?,
        })
    }

    pub fn to_object(&self) -> AnonObject {
        let mut obj = AnonObject::new();
        obj.set(&Rc::from("indexed"), UniLValue::Object(Rc::new(RefCell::new(self.indexed.clone().into()))));
        obj.set(&Rc::from("lengths"), UniLValue::Object(Rc::new(RefCell::new(self.lengths.clone().into()))));
        obj
    }
}