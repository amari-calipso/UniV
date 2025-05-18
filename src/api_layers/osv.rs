use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{api_def_fn, api_field_def_fn, api_field_typedef_fn, api_fn_body, api_layer, api_typedef_fn, compiler::type_system::UniLType, univm::object::{AnonObject, UniLValue}};

api_fn_body! {
    default_timer(_args, [], _univ, _task) -> (UniLType::Float) {
        Ok(UniLValue::Float(now_as_secs()))
    }
}

api_fn_body! {
    timer(args, [UniLType::Any, UniLType::Float], univ, _task) -> (UniLType::Null) {
        let t = expect_float(&args[1], "first argument of 'sortingVisualizer.timer'", univ)?;
        univ.time += now_as_secs() - t;
        Ok(UniLValue::Null)
    }
}

api_fn_body! {
    write(args, [UniLType::Any, UniLType::List, integer_type(), UniLType::Any], univ, task) -> (UniLType::Null) {
        args.remove(0);
        crate::api_layers::univ::UniV_invisibleWrite::func(univ, args, task)
    }
}

api_fn_body! {
    swap(args, [UniLType::Any, UniLType::List, integer_type(), integer_type()], univ, task) -> (UniLType::Null) {
        args.remove(0);
        crate::api_layers::univ::UniV_invisibleSwap::func(univ, args, task)
    }
}

api_fn_body! {
    highlight(args, [UniLType::Any, integer_type()], univ, task) -> (UniLType::Null) {
        args.remove(0);
        crate::api_layers::univ::UniV_immediateHighlight::func(univ, args, task)
    }
}

api_fn_body! {
    multiHighlight(args, [UniLType::Any, UniLType::List], univ, task) -> (UniLType::Null) {
        args.remove(0);
        crate::api_layers::univ::UniV_immediateMultiHighlight::func(univ, args, task)
    }
}

api_fn_body! {
    highlightAdvanced(args, [UniLType::Any, object_type()], univ, task) -> (UniLType::Null) {
        args.remove(0);
        crate::api_layers::univ::UniV_immediateHighlightAdvanced::func(univ, args, task)
    }
}

api_fn_body! {
    multiHighlightAdvanced(args, [UniLType::Any, UniLType::List], univ, task) -> (UniLType::Null) {
        args.remove(0);
        crate::api_layers::univ::UniV_immediateMultiHighlightAdvanced::func(univ, args, task)
    }
}

api_fn_body! {
    queueHighlight(args, [UniLType::Any, integer_type()], univ, task) -> (UniLType::Null) {
        args.remove(0);
        crate::api_layers::univ::UniV_highlight::func(univ, args, task)
    }
}

api_fn_body! {
    queueMultiHighlight(args, [UniLType::Any, UniLType::List], univ, task) -> (UniLType::Null) {
        args.remove(0);
        crate::api_layers::univ::UniV_multiHighlight::func(univ, args, task)
    }
}

api_fn_body! {
    queueHighlightAdvanced(args, [UniLType::Any, object_type()], univ, task) -> (UniLType::Null) {
        args.remove(0);
        crate::api_layers::univ::UniV_highlightAdvanced::func(univ, args, task)
    }
}

api_fn_body! {
    queueMultiHighlightAdvanced(args, [UniLType::Any, UniLType::List], univ, task) -> (UniLType::Null) {
        args.remove(0);
        crate::api_layers::univ::UniV_multiHighlightAdvanced::func(univ, args, task)
    }
}

api_fn_body! {
    markArray(args, [UniLType::Any, UniLType::Int, integer_type()], univ, task) -> (UniLType::Null) {
        args.remove(0);
        crate::api_layers::univ::UniV_markArray::func(univ, args, task)
    }
}

api_fn_body! {
    markArrayAdvanced(args, [UniLType::Any, UniLType::Int, object_type()], univ, task) -> (UniLType::Null) {
        args.remove(0);
        crate::api_layers::univ::UniV_markArrayAdvanced::func(univ, args, task)
    }
}

api_fn_body! {
    clearMark(args, [UniLType::Any, UniLType::Int], univ, task) -> (UniLType::Null) {
        args.remove(0);
        crate::api_layers::univ::UniV_clearMark::func(univ, args, task)
    }
}

api_fn_body! {
    clearAllMarks(args, [UniLType::Any], univ, task) -> (UniLType::Null) {
        args.remove(0);
        crate::api_layers::univ::UniV_clearAllMarks::func(univ, args, task)
    }
}

api_fn_body! {
    createValueArray(args, [UniLType::Any, UniLType::Int], univ, task) -> (UniLType::List) {
        args.remove(0);
        crate::api_layers::univ::Array::func(univ, args, task)
    }
}

api_fn_body! {
    removeAux(args, [UniLType::Any, UniLType::List], univ, task) -> (UniLType::Null) {
        args.remove(0);
        crate::api_layers::univ::UniV_removeAux::func(univ, args, task)
    }
}

api_fn_body! {
    addAux(args, [UniLType::Any, UniLType::List], univ, task) -> (UniLType::Null) {
        args.remove(0);
        crate::api_layers::univ::UniV_addAux::func(univ, args, task)
    }
}

api_fn_body! {
    setNonOrigAux(args, [UniLType::Any, UniLType::List], univ, task) -> (UniLType::Null) {
        let obj = expect_object(&args[1], "first argument of 'sortingVisualizer.setNonOrigAux'", univ)?;

        if let AnyObject::List(list) = &*obj.borrow() {
            for item in &list.items {
                crate::api_layers::univ::UniV_setNonOrigAux::func(univ, vec![item.clone()], task)?;
            }

            Ok(UniLValue::Null)
        } else {
            Err(univ.vm.create_exception(UniLValue::String(format!(
                "First argument of 'sortingVisualizer.setNonOrigAux' must be a list (got {})",
                args[1].stringify_type()
            ).into())))
        }
    }
}

api_fn_body! {
    setInvisibleArray(args, [UniLType::Any, UniLType::Any], _univ, _task) {
        Ok(args[1].clone())
    }
}

api_fn_body! {
    getPivotSelectionByName(args, [UniLType::Any, UniLType::String], univ, task) {
        args.remove(0);
        crate::api_layers::univ::UniV_getPivotSelection::func(univ, args, task)
    }
}

api_fn_body! {
    getPivotSelectionByID(args, [UniLType::Any, UniLType::Int], univ, _task) {
        let id = expect_int_strict(&args[1], "first argument of 'sortingVisualizer.getPivotSelectionByID'", univ)?;
        let name = Rc::clone(&univ.gui.pivot_selections[id as usize]);
        Ok(UniLValue::Object(Rc::new(RefCell::new(univ.get_pivot_selection(&name)?.clone().into()))))
    }
}

api_fn_body! {
    getPivotSelectionsNames(_args, [UniLType::Any], univ, _task) {
        let list = List::from(univ.gui.pivot_selections.iter().map(|x| UniLValue::String(Rc::clone(&x))).collect());
        Ok(UniLValue::Object(Rc::new(RefCell::new(list.into()))))
    }
}

api_fn_body! {
    getRotationByName(args, [UniLType::Any, UniLType::String], univ, task) -> (object_type()) {
        args.remove(0);
        crate::api_layers::univ::UniV_getRotation::func(univ, args, task)
    }
}

api_fn_body! {
    getRotationByID(args, [UniLType::Any, UniLType::Int], univ, _task) -> (object_type()) {
        let id = expect_int_strict(&args[1], "first argument of 'sortingVisualizer.getRotationByID'", univ)?;
        let name = Rc::clone(&univ.gui.rotations[id as usize]);
        Ok(UniLValue::Object(Rc::new(RefCell::new(univ.get_rotation(&name)?.to_object().into()))))
    }
}

api_fn_body! {
    getRotationsNames(_args, [UniLType::Any], univ, _task) {
        let list = List::from(univ.gui.rotations.iter().map(|x| UniLValue::String(Rc::clone(&x))).collect());
        Ok(UniLValue::Object(Rc::new(RefCell::new(list.into()))))
    }
}

api_fn_body! {
    setSpeed(args, [UniLType::Any, UniLType::Float], univ, task) -> (UniLType::Null) {
        args.remove(0);
        crate::api_layers::univ::UniV_setSpeed::func(univ, args, task)
    }
}

api_fn_body! {
    resetSpeed(args, [UniLType::Any], univ, task) -> (UniLType::Null) {
        crate::api_layers::univ::UniV_resetSpeed::func(univ, args, task)
    }
}

api_fn_body! {
    getSpeed(args, [UniLType::Any], univ, task) -> (UniLType::Float) {
        crate::api_layers::univ::UniV_getSpeed::func(univ, args, task)
    }
}

api_fn_body! {
    delay(args, [UniLType::Any, exclusive_number_type()], univ, task) -> (UniLType::Null) {
        args.remove(0);
        crate::api_layers::univ::UniV_delay::func(univ, args, task)
    }
}

api_fn_body! {
    setCurrentlyRunning(args, [UniLType::Any, UniLType::String], univ, task) -> (UniLType::Null) {
        args.remove(0);
        crate::api_layers::univ::UniV_setCurrentlyRunning::func(univ, args, task)
    }
}

api_fn_body! {
    getUserSelection(args, [UniLType::Any, UniLType::List, UniLType::String], univ, task) -> (UniLType::Int) {
        args.remove(0);
        crate::api_layers::univ::UniV_getUserSelection::func(univ, args, task)
    }
}

api_fn_body! {
    getUserInput(
        args, 
        [UniLType::Any]
            .into_iter()
            .chain(crate::api_layers::univ::UniV_getUserInput::args().into_iter())
            .collect::<Vec<UniLType>>(),
        univ, task
    ) {
        args.remove(0);
        crate::api_layers::univ::UniV_getUserInput::func(univ, args, task)
    }
}

api_fn_body! {
    userWarn(args, [UniLType::Any, UniLType::String], univ, task) -> (UniLType::Null) {
        args.remove(0);
        crate::api_layers::univ::UniV_popup::func(univ, args, task)
    }
}

api_fn_body! {
    pushAutoValue(args, [UniLType::Any, UniLType::Any], univ, task) -> (UniLType::Null) {
        args.remove(0);
        crate::api_layers::univ::UniV_pushAutoValue::func(univ, args, task)
    }
}

api_fn_body! {
    popAutoValue(args, [UniLType::Any], univ, task) {
        crate::api_layers::univ::UniV_popAutoValue::func(univ, args, task)
    }
}

api_fn_body! {
    resetAutoValues(args, [UniLType::Any], univ, task) -> (UniLType::Null) {
        crate::api_layers::univ::UniV_resetAutoValues::func(univ, args, task)
    }
}

api_fn_body! {
    HighlightInfo(
        args, [
            UniLType::Int, 
            nullable_type(UniLType::List),
            nullable_type(api_layers::univ::color_type()),
            UniLType::Any,
            UniLType::Any
        ], 
        univ, _task
    ) -> (UniLType::Null) {
        expect_int(&args[0], "first argument of 'HighlightInfo'", univ)?;

        if !matches!(args[1], UniLValue::Null) {
            let obj = expect_object(&args[1], "second argument of 'HighlightInfo'", univ)?;
            if !matches!(&*obj.borrow(), AnyObject::List(_)) {
                return Err(univ.vm.create_exception(UniLValue::String(format!(
                    "Expecting List as second argument of 'HighlightInfo' but got {}",
                    args[1].stringify_type()
                ).into())));
            }
        }

        if !matches!(args[2], UniLValue::Null) {
            let obj = expect_object(&args[2], "third argument of 'HighlightInfo'", univ)?;

            if let AnyObject::AnonObject(instance) = &*obj.borrow() {
                if !(
                    instance.fields.contains_key("r") &&
                    instance.fields.contains_key("g") &&
                    instance.fields.contains_key("b")
                ) {
                    return Err(univ.vm.create_exception(UniLValue::String(
                        Rc::from("Color object for highlight is not properly formed")
                    )));
                }
            } else {
                return Err(univ.vm.create_exception(UniLValue::String(format!(
                    "Expecting Color object as second argument of 'HighlightInfo' but got {}",
                    args[2].stringify_type()
                ).into())));
            }
        }

        let mut obj = AnonObject::new();
        obj.set(&Rc::from("idx"), args[0].clone());
        obj.set(&Rc::from("aux"), args[1].clone());
        obj.set(&Rc::from("color"), args[2].clone());
        obj.set(&Rc::from("silent"), args[3].clone());
        obj.set(&Rc::from("isWrite"), args[4].clone());
        Ok(UniLValue::Object(Rc::new(RefCell::new(obj.into()))))
    }
}

api_layer! {
    definitions(globals) {
        api_def_fn!(globals, default_timer);
        api_def_fn!(globals, HighlightInfo);

        let mut sv = AnonObject::new();
        api_field_def_fn!(sv, timer);
        api_field_def_fn!(sv, write);
        api_field_def_fn!(sv, swap);

        api_field_def_fn!(sv, highlight);
        api_field_def_fn!(sv, multiHighlight);
        api_field_def_fn!(sv, highlightAdvanced);
        api_field_def_fn!(sv, multiHighlightAdvanced);
        api_field_def_fn!(sv, queueHighlight);
        api_field_def_fn!(sv, queueMultiHighlight);
        api_field_def_fn!(sv, queueHighlightAdvanced);
        api_field_def_fn!(sv, queueMultiHighlightAdvanced);
        api_field_def_fn!(sv, markArray);
        api_field_def_fn!(sv, markArrayAdvanced);
        api_field_def_fn!(sv, clearMark);
        api_field_def_fn!(sv, clearAllMarks);

        api_field_def_fn!(sv, createValueArray);
        api_field_def_fn!(sv, removeAux);
        api_field_def_fn!(sv, addAux);
        api_field_def_fn!(sv, setNonOrigAux);
        api_field_def_fn!(sv, setInvisibleArray);

        api_field_def_fn!(sv, getPivotSelectionByName);
        api_field_def_fn!(sv, getPivotSelectionByID);
        api_field_def_fn!(sv, getPivotSelectionsNames);
        api_field_def_fn!(sv, getRotationByName);
        api_field_def_fn!(sv, getRotationByID);
        api_field_def_fn!(sv, getRotationsNames);

        api_field_def_fn!(sv, setSpeed);
        api_field_def_fn!(sv, resetSpeed);
        api_field_def_fn!(sv, getSpeed);
        api_field_def_fn!(sv, delay);

        api_field_def_fn!(sv, setCurrentlyRunning);
        api_field_def_fn!(sv, getUserSelection);
        api_field_def_fn!(sv, getUserInput);
        api_field_def_fn!(sv, userWarn);
        api_field_def_fn!(sv, pushAutoValue);
        api_field_def_fn!(sv, popAutoValue);
        api_field_def_fn!(sv, resetAutoValues);

        globals.define(&Rc::from("sortingVisualizer"), UniLValue::Object(Rc::new(RefCell::new(sv.into()))));
    }

    types(globals) {
        api_typedef_fn!(globals, default_timer);
        api_typedef_fn!(globals, HighlightInfo);

        let mut sv = HashMap::new();
        api_field_typedef_fn!(sv, timer);
        api_field_typedef_fn!(sv, write);
        api_field_typedef_fn!(sv, swap);

        api_field_typedef_fn!(sv, highlight);
        api_field_typedef_fn!(sv, multiHighlight);
        api_field_typedef_fn!(sv, highlightAdvanced);
        api_field_typedef_fn!(sv, multiHighlightAdvanced);
        api_field_typedef_fn!(sv, queueHighlight);
        api_field_typedef_fn!(sv, queueMultiHighlight);
        api_field_typedef_fn!(sv, queueHighlightAdvanced);
        api_field_typedef_fn!(sv, queueMultiHighlightAdvanced);
        api_field_typedef_fn!(sv, markArray);
        api_field_typedef_fn!(sv, markArrayAdvanced);
        api_field_typedef_fn!(sv, clearMark);
        api_field_typedef_fn!(sv, clearAllMarks);

        api_field_typedef_fn!(sv, createValueArray);
        api_field_typedef_fn!(sv, removeAux);
        api_field_typedef_fn!(sv, addAux);
        api_field_typedef_fn!(sv, setNonOrigAux);
        api_field_typedef_fn!(sv, setInvisibleArray);

        api_field_typedef_fn!(sv, getPivotSelectionByName);
        api_field_typedef_fn!(sv, getPivotSelectionByID);
        api_field_typedef_fn!(sv, getPivotSelectionsNames);
        api_field_typedef_fn!(sv, getRotationByName);
        api_field_typedef_fn!(sv, getRotationByID);
        api_field_typedef_fn!(sv, getRotationsNames);

        api_field_typedef_fn!(sv, setSpeed);
        api_field_typedef_fn!(sv, resetSpeed);
        api_field_typedef_fn!(sv, getSpeed);
        api_field_typedef_fn!(sv, delay);

        api_field_typedef_fn!(sv, setCurrentlyRunning);
        api_field_typedef_fn!(sv, getUserSelection);
        api_field_typedef_fn!(sv, getUserInput);
        api_field_typedef_fn!(sv, userWarn);
        api_field_typedef_fn!(sv, pushAutoValue);
        api_field_typedef_fn!(sv, popAutoValue);
        api_field_typedef_fn!(sv, resetAutoValues);

        globals.define(&Rc::from("sortingVisualizer"), UniLType::Object { fields: Rc::new(RefCell::new(sv)) });
    }
}