use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::{api_layer_fn, compiler::type_system::UniLType};

api_layer_fn! {
    Color(args, [UniLType::Int, UniLType::Int, UniLType::Int], univ, _task) -> (color_type()) {
        let mut obj = AnonObject::new();
        obj.set(&Rc::from("r"), UniLValue::Int(expect_int_range_strict(&args[0], 0..256, "first argument of 'Color'", univ)?));
        obj.set(&Rc::from("g"), UniLValue::Int(expect_int_range_strict(&args[1], 0..256, "second argument of 'Color'", univ)?));
        obj.set(&Rc::from("b"), UniLValue::Int(expect_int_range_strict(&args[2], 0..256, "third argument of 'Color'", univ)?));
        Ok(UniLValue::Object(Rc::new(RefCell::new(obj.into()))))
    }

    Array(args, [UniLType::Int], univ, _task) -> (UniLType::List) {
        let length = expect_int_range_strict(&args[0], 0..i64::MAX, "first argument of 'Array'", univ)?;
        Ok(univ.create_aux(length as usize))
    }

    StandaloneArray(args, [UniLType::Int], univ, _task) -> (UniLType::List) {
        let length = expect_int_range_strict(&args[0], 0..i64::MAX, "first argument of 'StandaloneArray'", univ)?;
        Ok(UniLValue::Object(univ.standalone_array(length as usize)))
    }

    UniV_immediateSort(args, [UniLType::List, UniLType::Int, UniLType::Int], univ, _task) -> (UniLType::List) {
        let array = expect_object(&args[0], "first argument of 'UniV_immediateSort'", univ)?;
        let s = expect_int_strict(&args[1], "second argument of 'UniV_immediateSort'", univ)?;
        let e = expect_int_strict(&args[2], "third argument of 'UniV_immediateSort'", univ)?;

        if !matches!(&*array.borrow(), AnyObject::List(_)) {
            return Err(univ.vm.create_exception(UniLValue::String(format!(
                "Expecting list as first argument of 'UniV_immediateSort' (got {})",
                args[1].stringify_type()
            ).into())));
        }

        let a;
        let b;

        {
            let list = array.borrow();
            a = expect_list!(list).convert_index(s);
            b = expect_list!(list).convert_index(e);
        }

        if a > b {
            return Err(univ.vm.create_exception(UniLValue::String(format!(
                "Start bound ({}) of sort cannot be greater than end bound ({})",
                s, e
            ).into())));
        }

        let mut tmp = Vec::with_capacity(b - a);

        {
            let list = array.borrow();
            for (i, item) in expect_list!(list).items.iter().skip(a).take(b - a).enumerate() {
                if let UniLValue::Value { value, idx } = item {
                    tmp.push(Value::new(*value, *idx, None));
                } else {
                    return Err(univ.vm.create_exception(UniLValue::String(format!(
                        "Item of type '{}' (at index {}) is not allowed in main array",
                        item.stringify_type(), i
                    ).into())));
                }
            }
        }

        tmp.sort();

        let id = univ.get_optional_aux_id(array.as_ptr() as *const AnyObject);
        for (i, item) in tmp.into_iter().enumerate() {
            {
                let mut list = array.borrow_mut();
                expect_list_mut!(list).items[a + i] = UniLValue::Value { value: item.value, idx: item.idx };
            }

            univ.immediate_highlight(vec![HighlightInfo::from_idx_and_aux_write(a + i, id)])?;
        }

        Ok(UniLValue::Object(Rc::clone(array)))
    }

    UniV_immediateHighlightAdvanced(args, [object_type()], univ, _task) -> (UniLType::Null) {
        let hinfo = HighlightInfo::from_obj(&args[0], univ)?;
        univ.immediate_highlight(vec![hinfo])?;
        Ok(UniLValue::Null)
    }

    UniV_immediateHighlight(args, [integer_type()], univ, _task) -> (UniLType::Null) {
        let idx = expect_int(&args[0], "first argument of 'UniV_immediateHighlight'", univ)?;
        univ.immediate_highlight(vec![HighlightInfo::from_idx(List::convert_index_for_len(univ.shared.array.len(), idx))])?;
        Ok(UniLValue::Null)
    }

    UniV_immediateHighlightAux(args, [integer_type(), UniLType::List], univ, _task) -> (UniLType::Null) {
        let idx = expect_int(&args[0], "first argument of 'UniV_immediateHighlightAux'", univ)?;
        let obj = expect_object(&args[1], "second argument of 'UniV_immediateHighlightAux'", univ)?;

        if let AnyObject::List(list) = &*obj.borrow() {
            let aux_id = univ.get_optional_aux_id(obj.as_ptr() as *const AnyObject);
            univ.immediate_highlight(vec![HighlightInfo::from_idx_and_aux(list.convert_index(idx), aux_id)])?;
            Ok(UniLValue::Null)
        } else {
            Err(univ.vm.create_exception(UniLValue::String(format!(
                "Auxiliary array for highlight must be a list (got {})",
                args[1].stringify_type()
            ).into())))
        }
    }

    UniV_immediateMultiHighlightAdvanced(args, [UniLType::List], univ, _task) -> (UniLType::Null) {
        if let AnyObject::List(highlights) = &*expect_object(&args[0], "first argument of 'UniV_immediateMultiHighlightAdvanced'", univ)?.borrow() {
            for highlight in &highlights.items {
                let hinfo = HighlightInfo::from_obj(highlight, univ)?;
                univ.highlights.push(hinfo);
            }

            univ.immediate_highlight(Vec::new())?;
            Ok(UniLValue::Null)
        } else {
            Err(univ.vm.create_exception(UniLValue::String(format!(
                "Expecting list as first argument of 'UniV_immediateMultiHighlightAdvanced' but got {}",
                args[0].stringify_type()
            ).into())))
        }
    }

    UniV_immediateMultiHighlight(args, [UniLType::List], univ, _task) -> (UniLType::Null) {
        if let AnyObject::List(highlights) = &*expect_object(&args[0], "first argument of 'UniV_immediateMultiHighlight'", univ)?.borrow() {
            for (i, highlight) in highlights.items.iter().enumerate() {
                let idx = expect_int(
                    highlight,
                    format!("item (at index {}) of list passed in first argument of 'UniV_immediateMultiHighlight'", i).as_str(),
                    univ
                )?;

                univ.highlights.push(HighlightInfo::from_idx(List::convert_index_for_len(univ.shared.array.len(), idx)));
            }

            univ.immediate_highlight(Vec::new())?;
            Ok(UniLValue::Null)
        } else {
            Err(univ.vm.create_exception(UniLValue::String(format!(
                "Expecting list as first argument of 'UniV_immediateMultiHighlight' but got {}",
                args[0].stringify_type()
            ).into())))
        }
    }

    UniV_highlightAdvanced(args, [object_type()], univ, _task) -> (UniLType::Null) {
        let hinfo = HighlightInfo::from_obj(&args[0], univ)?;
        univ.highlights.push(hinfo);
        Ok(UniLValue::Null)
    }

    UniV_highlight(args, [integer_type()], univ, _task) -> (UniLType::Null) {
        let idx = expect_int(&args[0], "first argument of 'UniV_highlight'", univ)?;
        univ.highlights.push(HighlightInfo::from_idx(List::convert_index_for_len(univ.shared.array.len(), idx)));
        Ok(UniLValue::Null)
    }

    UniV_highlightAux(args, [integer_type(), UniLType::List], univ, _task) -> (UniLType::Null) {
        let idx = expect_int(&args[0], "first argument of 'UniV_highlightAux'", univ)?;
        let obj = expect_object(&args[1], "second argument of 'UniV_highlightAux'", univ)?;

        if let AnyObject::List(list) = &*obj.borrow() {
            let aux_id = univ.get_optional_aux_id(obj.as_ptr() as *const AnyObject);
            univ.highlights.push(HighlightInfo::from_idx_and_aux(list.convert_index(idx), aux_id));
            Ok(UniLValue::Null)
        } else {
            Err(univ.vm.create_exception(UniLValue::String(format!(
                "Auxiliary array for highlight must be a list (got {})",
                args[1].stringify_type()
            ).into())))
        }
    }

    UniV_multiHighlightAdvanced(args, [UniLType::List], univ, _task) -> (UniLType::Null) {
        if let AnyObject::List(highlights) = &*expect_object(&args[0], "first argument of 'UniV_multiHighlightAdvanced'", univ)?.borrow() {
            for highlight in &highlights.items {
                let hinfo = HighlightInfo::from_obj(highlight, univ)?;
                univ.highlights.push(hinfo);
            }

            Ok(UniLValue::Null)
        } else {
            Err(univ.vm.create_exception(UniLValue::String(format!(
                "Expecting list as first argument of 'UniV_multiHighlightAdvanced' but got {}",
                args[1].stringify_type()
            ).into())))
        }
    }

    UniV_multiHighlight(args, [UniLType::List], univ, _task) -> (UniLType::Null) {
        if let AnyObject::List(highlights) = &*expect_object(&args[0], "first argument of 'UniV_multiHighlight'", univ)?.borrow() {
            for (i, highlight) in highlights.items.iter().enumerate() {
                let idx = expect_int(
                    highlight,
                    format!("item (at index {}) of list passed in first argument of 'UniV_multiHighlight'", i).as_str(),
                    univ
                )?;

                univ.highlights.push(HighlightInfo::from_idx(List::convert_index_for_len(univ.shared.array.len(), idx)));
            }

            Ok(UniLValue::Null)
        } else {
            Err(univ.vm.create_exception(UniLValue::String(format!(
                "Expecting list as first argument of 'UniV_multiHighlight' but got {}",
                args[1].stringify_type()
            ).into())))
        }
    }

    UniV_markArray(args, [UniLType::Int, integer_type()], univ, _task) -> (UniLType::Null) {
        let id = expect_int_range_strict(&args[0], 0..i64::MAX, "first argument of 'UniV_markArray'", univ)? as usize;
        let idx = expect_int(&args[1], "second argument of 'UniV_markArray'", univ)?;
        univ.marks.insert(id, HighlightInfo::from_idx(List::convert_index_for_len(univ.shared.array.len(), idx)));
        Ok(UniLValue::Null)
    }

    UniV_markArrayAux(args, [UniLType::Int, integer_type(), UniLType::List], univ, _task) -> (UniLType::Null) {
        let id = expect_int_range_strict(&args[0], 0..i64::MAX, "first argument of 'UniV_markArrayAux'", univ)? as usize;
        let idx = expect_int(&args[1], "second argument of 'UniV_markArrayAux'", univ)?;
        let obj = expect_object(&args[2], "third argument of 'UniV_markArrayAux'", univ)?;

        if let AnyObject::List(list) = &*obj.borrow() {
            let aux_id = univ.get_optional_aux_id(obj.as_ptr() as *const AnyObject);
            univ.marks.insert(id, HighlightInfo::from_idx_and_aux(list.convert_index(idx), aux_id));
            Ok(UniLValue::Null)
        } else {
            Err(univ.vm.create_exception(UniLValue::String(format!(
                "Auxiliary array for highlight must be a list (got {})",
                args[1].stringify_type()
            ).into())))
        }
    }

    UniV_markArrayAdvanced(args, [UniLType::Int, object_type()], univ, _task) -> (UniLType::Null) {
        let id = expect_int_range_strict(&args[0], 0..i64::MAX, "first argument of 'UniV_markArrayAdvanced'", univ)? as usize;
        let hinfo = HighlightInfo::from_obj(&args[1], univ)?;
        univ.marks.insert(id, hinfo);
        Ok(UniLValue::Null)
    }

    UniV_clearMark(args, [UniLType::Int], univ, _task) -> (UniLType::Null) {
        let id = expect_int_range_strict(&args[0], 0..i64::MAX, "first argument of 'UniV_clearMark'", univ)? as usize;
        univ.marks.remove(&id);
        Ok(UniLValue::Null)
    }

    UniV_clearAllMarks(_args, [], univ, _task) -> (UniLType::Null) {
        univ.marks.clear();
        Ok(UniLValue::Null)
    }

    UniV_removeAux(args, [UniLType::List], univ, _task) -> (UniLType::Null) {
        let obj = expect_object(&args[0], "first argument of 'UniV_removeAux'", univ)?;

        if !matches!(&*obj.borrow(), AnyObject::List(_)) {
            return Err(univ.vm.create_exception(UniLValue::String(format!(
                "Auxiliary to remove must be a list (got {})",
                args[0].stringify_type()
            ).into())));
        }

        univ.remove_aux(obj);
        Ok(UniLValue::Null)
    }

    UniV_addAux(args, [UniLType::List], univ, _task) -> (UniLType::Null) {
        let obj = expect_object(&args[0], "first argument of 'UniV_addAux'", univ)?;

        if !matches!(&*obj.borrow(), AnyObject::List(_)) {
            return Err(univ.vm.create_exception(UniLValue::String(format!(
                "Auxiliary to add must be a list (got {})",
                args[0].stringify_type()
            ).into())));
        }

        univ.add_aux(obj)?;
        Ok(UniLValue::Null)
    }

    UniV_setNonOrigAux(args, [UniLType::List], univ, _task) -> (UniLType::Null) {
        let obj = expect_object(&args[0], "first argument of 'UniV_setNonOrigAux'", univ)?;

        if !matches!(&*obj.borrow(), AnyObject::List(_)) {
            return Err(univ.vm.create_exception(UniLValue::String(format!(
                "Auxiliary to set as non original must be a list (got {})",
                args[0].stringify_type()
            ).into())));
        }

        univ.set_non_orig_aux(obj);
        Ok(UniLValue::Null)
    }

    UniV_getPivotSelection(args, [UniLType::String], univ, _task) {
        let name = expect_string(&args[0], "first argument of 'UniV_getPivotSelection'", univ)?;
        Ok(UniLValue::Object(Rc::new(RefCell::new(univ.get_pivot_selection(&name)?.clone().into()))))
    }

    UniV_getUserPivotSelection(args, [UniLType::String, UniLType::Group(HashSet::from([UniLType::String, UniLType::Null]))], univ, _task) {
        let msg = expect_string(&args[0], "first argument of 'UniV_getUserPivotSelection'", univ)?;
        let default = expect_optional_string(&args[1], "second argument of 'UniV_getUserPivotSelection'", univ)?;

        let popped = univ.pop_autovalue();
        match popped {
            UniLValue::Null => (),
            UniLValue::String(name) => {
                return Ok(UniLValue::Object(Rc::new(RefCell::new(
                    univ.get_pivot_selection(&name)?.clone().into()
                ))));
            }
            _ => {
                return Err(univ.vm.create_exception(UniLValue::String(format!(
                    "Autovalue for UniV_getUserPivotSelection must be String but got {}",
                    popped.stringify_type()
                ).into())));
            }
        }

        let pivot_selections = univ.gui.pivot_selections.clone();
        let mut default_idx = 0;
        let mut full_msg = String::from(msg.as_ref());
        
        if let Some(default) = default {
            full_msg.push_str(" (default: ");
            full_msg.push_str(&default);
            full_msg.push(')');

            if let Ok(index) = pivot_selections.binary_search(&default) {
                default_idx = index;
            } else {
                return Err(univ.vm.create_exception(UniLValue::String(format!(
                    "Unknown default pivot selection \"{}\"", default
                ).into())));
            }
        }

        univ.save_background();
        univ.gui.build_fn = Gui::selection;
        univ.gui.selection.set(&univ.currently_running, &full_msg, pivot_selections, default_idx)
            .map_err(|e| univ.vm.create_exception(UniLValue::String(e)))?;
        univ.run_gui()?;

        let name = Rc::clone(&univ.gui.pivot_selections[univ.gui.selection.index]);
        let ret = Ok(UniLValue::Object(Rc::new(RefCell::new(univ.get_pivot_selection(&name)?.clone().into()))));

        if univ.store_user_values {
            univ.user_values.push(UniLValue::String(name));
        }

        ret
    }

    UniV_getRotation(args, [UniLType::String], univ, _task) -> (object_type()) {
        let name = expect_string(&args[0], "first argument of 'UniV_getRotation'", univ)?;
        Ok(UniLValue::Object(Rc::new(RefCell::new(univ.get_rotation(&name)?.to_object().into()))))
    }

    UniV_getUserRotation(args, [UniLType::String, UniLType::Group(HashSet::from([UniLType::String, UniLType::Null]))], univ, _task) {
        let msg = expect_string(&args[0], "first argument of 'UniV_getUserRotation'", univ)?;
        let default = expect_optional_string(&args[1], "second argument of 'UniV_getUserRotation'", univ)?;

        let popped = univ.pop_autovalue();
        match popped {
            UniLValue::Null => (),
            UniLValue::String(name) => {
                return Ok(UniLValue::Object(Rc::new(RefCell::new(
                    univ.get_rotation(&name)?.to_object().into()
                ))));
            }
            _ => {
                return Err(univ.vm.create_exception(UniLValue::String(format!(
                    "Autovalue for UniV_getUserRotation must be String but got {}",
                    popped.stringify_type()
                ).into())));
            }
        }

        let rotations = univ.gui.rotations.clone();
        let mut default_idx = 0;
        let mut full_msg = String::from(msg.as_ref());
        
        if let Some(default) = default {
            full_msg.push_str(" (default: ");
            full_msg.push_str(&default);
            full_msg.push(')');

            if let Ok(index) = rotations.binary_search(&default) {
                default_idx = index;
            } else {
                return Err(univ.vm.create_exception(UniLValue::String(format!(
                    "Unknown default rotation \"{}\"", default
                ).into())));
            }
        }

        univ.save_background();
        univ.gui.build_fn = Gui::selection;
        univ.gui.selection.set(&univ.currently_running, &full_msg, rotations, default_idx)
            .map_err(|e| univ.vm.create_exception(UniLValue::String(e)))?;
        univ.run_gui()?;

        let name = Rc::clone(&univ.gui.rotations[univ.gui.selection.index]);
        let ret = Ok(UniLValue::Object(Rc::new(RefCell::new(univ.get_rotation(&name)?.to_object().into()))));

        if univ.store_user_values {
            univ.user_values.push(UniLValue::String(name));
        }

        ret
    }

    UniV_setSpeed(args, [exclusive_number_type()], univ, _task) -> (UniLType::Null) {
        let speed = expect_number(&args[0], "first argument of 'UniV_setSpeed'", univ)?;
        univ.set_speed(speed)
            .map_err(|e| univ.vm.create_exception(UniLValue::String(e)))?;
        Ok(UniLValue::Null)
    }

    UniV_resetSpeed(_args, [], univ, _task) -> (UniLType::Null) {
        univ.reset_speed();
        Ok(UniLValue::Null)
    }

    UniV_getSpeed(_args, [], univ, _task) -> (UniLType::Float) {
        Ok(UniLValue::Float(univ.get_speed()))
    }

    UniV_delay(args, [exclusive_number_type()], univ, _task) -> (UniLType::Null) {
        let delay = expect_number(&args[0], "first argument of 'UniV_delay'", univ)?;
        univ.delay(delay);
        Ok(UniLValue::Null)
    }

    UniV_setCurrentlyRunning(args, [UniLType::String], univ, _task) -> (UniLType::Null) {
        let name = expect_string(&args[0], "first argument of 'UniV_setCurrentlyRunning'", univ)?;
        univ.set_current_name(&name);
        Ok(UniLValue::Null)
    }

    UniV_getUserSelection(args, [UniLType::List, UniLType::String], univ, _task) -> (UniLType::Int) {
        let obj = expect_object(&args[0], "first argument of 'UniV_getUserSelection'", univ)?;
        let msg = expect_string(&args[1], "second argument of 'UniV_getUserSelection'", univ)?;

        if let AnyObject::List(list) = &*obj.borrow() {
            let popped = univ.pop_autovalue();
            if !matches!(popped, UniLValue::Null) {
                return Ok(popped);
            }

            let mut selection_list = Vec::with_capacity(list.items.len());
            for (i, item) in list.items.iter().enumerate() {
                selection_list.push(expect_string(item, format!("element at index {} of selection list", i).as_str(), univ)?);
            }

            univ.save_background();
            univ.gui.build_fn = Gui::selection;
            univ.gui.selection.set(&univ.currently_running, &msg, selection_list, 0)
                .map_err(|e| univ.vm.create_exception(UniLValue::String(e)))?;
            univ.run_gui()?;

            if univ.store_user_values {
                univ.user_values.push(UniLValue::Int(univ.gui.selection.index as i64));
            }

            Ok(UniLValue::Int(univ.gui.selection.index as i64))
        } else {
            Err(univ.vm.create_exception(UniLValue::String(format!(
                "Selection list must be a list (got {})",
                args[0].stringify_type()
            ).into())))
        }
    }

    UniV_getUserInput(
        args, [
            UniLType::String, 
            UniLType::String, 
            UniLType::Callable { 
                args: vec![UniLType::String], 
                return_type: Box::new(UniLType::Any)
            }
        ], 
        univ, task
    ) {
        let msg = expect_string(&args[0], "first argument of 'UniV_getUserInput'", univ)?;
        let default = expect_string(&args[1], "second argument of 'UniV_getUserInput'", univ)?;
        let obj = expect_object(&args[2], "third argument of 'UniV_getUserInput'", univ)?;

        if let AnyObject::AnyCallable(callable) = &*obj.borrow() {
            if callable.arity() != 1 {
                return Err(univ.vm.create_exception(UniLValue::String(format!(
                    "Expecting 1 parameter for callable at third argument but got {}",
                    callable.arity()
                ).into())));
            }

            if matches!(callable, AnyCallable::Function(_)) {
                return Err(univ.vm.create_exception(UniLValue::String(
                    Rc::from("Third argument of 'UniV_getUserInput' cannot be a user-defined function")
                )));
            }

            let popped = univ.pop_autovalue();
            if !matches!(popped, UniLValue::Null) {
                return Ok(popped);
            }

            let result;

            loop {
                univ.save_background();
                univ.gui.build_fn = Gui::text_input;
                univ.gui.text_input.set(&univ.currently_running, &msg, &default)
                    .map_err(|e| univ.vm.create_exception(UniLValue::String(e)))?;
                univ.run_gui()?;
                
                if let Ok(value) = callable.call(univ, vec![UniLValue::String(univ.gui.text_input.input.clone().into())], task) {
                    result = value;
                    break;
                }

                univ.gui.build_fn = Gui::popup;
                univ.gui.popup.set("Error", "Invalid input. Please retry").unwrap();
                univ.run_gui()?;
            }
            
            if univ.store_user_values {
                univ.user_values.push(result.clone());
            }

            Ok(result)
        } else {
            Err(univ.vm.create_exception(UniLValue::String(format!(
                "Third argument must be a callable (got {})",
                args[2].stringify_type()
            ).into())))
        }
    }

    UniV_popup(args, [UniLType::String], univ, _task) -> (UniLType::Null) {
        let msg = expect_string(&args[0], "first argument of 'UniV_popup'", univ)?;

        univ.save_background();
        univ.gui.build_fn = Gui::popup;
        univ.gui.popup.set(&univ.currently_running, &msg)
            .map_err(|e| univ.vm.create_exception(UniLValue::String(e)))?;
        univ.run_gui()?;

        Ok(UniLValue::Null)
    }

    UniV_pushAutoValue(args, [UniLType::Any], univ, _task) -> (UniLType::Null) {
        univ.push_autovalue(args[0].clone());
        Ok(UniLValue::Null)
    }

    UniV_popAutoValue(_args, [], univ, _task) {
        Ok(univ.pop_autovalue())
    }

    UniV_resetAutoValues(_args, [], univ, _task) -> (UniLType::Null) {
        univ.reset_autovalues();
        Ok(UniLValue::Null)
    }

    UniV_invisibleWrite(args, [UniLType::List, integer_type(), UniLType::Any], univ, _task) -> (UniLType::Null) {
        let array = expect_object(&args[0], "first argument of 'UniV_invisibleWrite'", univ)?;
        let i = expect_int(&args[1], "second argument of 'UniV_invisibleWrite'", univ)?;
        
        if let AnyObject::List(list) = &mut *array.borrow_mut() {
            with_timer!(univ, list.set_with_exception(i, args[2].clone(), univ)?);

            if univ.get_optional_aux_id(array.as_ptr() as *const AnyObject).is_none() {
                univ.main_stats.writes += 1;
            } else {
                univ.aux_stats.writes += 1;
            }
        } else {
            return Err(univ.vm.create_exception(UniLValue::String(format!(
                "Expecting list as first argument of 'UniV_invisibleWrite' (got {})",
                args[0].stringify_type()
            ).into())));
        }

        Ok(UniLValue::Null)
    }

    UniV_invisibleSwap(args, [UniLType::List, integer_type(), integer_type()], univ, _task) -> (UniLType::Null) {
        let obj = expect_object(&args[0], "first argument of 'UniV_invisibleSwap'", univ)?;

        if let AnyObject::List(list) = &mut *obj.borrow_mut() {
            let a = expect_int(&args[1], "second argument of 'UniV_invisibleSwap'", univ)?;
            let b = expect_int(&args[2], "third argument of 'UniV_invisibleSwap'", univ)?;

            let fail = univ.settings.unreliability.enabled && univ.rng.random_bool(univ.settings.unreliability.swaps);

            let left;
            if fail {
                // don't swap, but still perform the same operations so you can track time properly
                with_timer!(univ, {
                    left = list.get_with_exception(a, univ)?;
                    let right = list.get_with_exception(b, univ)?;
                    list.set(a, left.clone()).unwrap();
                    list.set(b, right).unwrap();
                });
            } else {
                with_timer!(univ, {
                    left = list.get_with_exception(a, univ)?;
                    let right = list.get_with_exception(b, univ)?;
                    list.set(a, right).unwrap();
                    list.set(b, left.clone()).unwrap();
                });
            }

            if univ.get_optional_aux_id(obj.as_ptr() as *const AnyObject).is_none() {
                univ.main_stats.failed_swaps += fail as u64;
                univ.main_stats.swaps  += 1;
                univ.main_stats.writes += 2;
            } else {
                univ.aux_stats.failed_swaps += fail as u64;
                univ.aux_stats.swaps  += 1;
                univ.aux_stats.writes += 2;
            }

            Ok(left)
        } else {
            Err(univ.vm.create_exception(UniLValue::String(format!(
                "Expecting list as first argument of 'UniV_invisibleSwap' but got {}",
                args[0].stringify_type()
            ).into())))
        }
    }

    UniV_invisibleRead(args, [UniLType::List, UniLType::Int], univ, _task) -> (UniLType::Any) {
        let array = expect_object(&args[0], "first argument of 'UniV_invisibleRead'", univ)?;
        let i = expect_int(&args[1], "second argument of 'UniV_invisibleRead'", univ)?;
        
        if let AnyObject::List(list) = &mut *array.borrow_mut() {
            let item = with_timer!(univ, list.get_with_exception(i, univ)?);

            if univ.get_optional_aux_id(array.as_ptr() as *const AnyObject).is_none() {
                univ.main_stats.reads += 1;
            } else {
                univ.aux_stats.reads += 1;
            }

            Ok(item)
        } else {
            Err(univ.vm.create_exception(UniLValue::String(format!(
                "Expecting list as first argument of 'UniV_invisibleRead' (got {})",
                args[0].stringify_type()
            ).into())))
        }
    }

    UniV_untrackedRead(args, [UniLType::List, UniLType::Int], univ, _task) -> (UniLType::Any) {
        let array = expect_object(&args[0], "first argument of 'UniV_untrackedRead'", univ)?;
        let i = expect_int(&args[1], "second argument of 'UniV_untrackedRead'", univ)?;
        
        if let AnyObject::List(list) = &mut *array.borrow_mut() {
            Ok(list.get_with_exception(i, univ)?)
        } else {
            Err(univ.vm.create_exception(UniLValue::String(format!(
                "Expecting list as first argument of 'UniV_untrackedRead' (got {})",
                args[0].stringify_type()
            ).into())))
        }
    }

    List_invisiblePush(args, [UniLType::List, UniLType::Any], univ, _task) -> (UniLType::Null) {
        let obj = expect_object(&args[0], "first argument of 'List_invisiblePush'", univ)?;

        if let AnyObject::List(list) = &mut *obj.borrow_mut() {
            with_timer!(univ, list.items.push(args[1].clone()));

            if univ.get_optional_aux_id(obj.as_ptr() as *const AnyObject).is_none() {
                univ.main_stats.writes += 1;
            } else {
                univ.aux_stats.writes += 1;
            }

            Ok(UniLValue::Null)
        } else {
            Err(univ.vm.create_exception(UniLValue::String(format!(
                "Expecting list as first argument of 'List_invisiblePush' but got {}",
                args[0].stringify_type()
            ).into())))
        }
    }

    UniV_addWrites(args, [UniLType::Int], univ, _task) -> (UniLType::Int) {
        let x = expect_int_strict(&args[0], "first argument of 'UniV_addWrites'", univ)?;
        univ.main_stats.writes = univ.main_stats.writes.saturating_add_signed(x);
        Ok(UniLValue::Int(univ.main_stats.writes as i64))
    }

    UniV_addReads(args, [UniLType::Int], univ, _task) -> (UniLType::Int) {
        let x = expect_int_strict(&args[0], "first argument of 'UniV_addReads'", univ)?;
        univ.main_stats.reads = univ.main_stats.reads.saturating_add_signed(x);
        Ok(UniLValue::Int(univ.main_stats.reads as i64))
    }

    UniV_addSwaps(args, [UniLType::Int], univ, _task) -> (UniLType::Int) {
        let x = expect_int_strict(&args[0], "first argument of 'UniV_addSwaps'", univ)?;
        univ.main_stats.swaps = univ.main_stats.swaps.saturating_add_signed(x);
        univ.main_stats.writes = univ.main_stats.writes.saturating_add_signed(x * 2);
        Ok(UniLValue::Int(univ.main_stats.swaps as i64))
    }

    UniV_addComparisons(args, [UniLType::Int], univ, _task) -> (UniLType::Int) {
        let x = expect_int_strict(&args[0], "first argument of 'UniV_addComparisons'", univ)?;
        univ.comparisons = univ.comparisons.saturating_add_signed(x);
        univ.main_stats.reads = univ.main_stats.reads.saturating_add_signed(x * 2);
        Ok(UniLValue::Int(univ.comparisons as i64))
    }

    UniV_addAuxWrites(args, [UniLType::Int], univ, _task) -> (UniLType::Int) {
        let x = expect_int_strict(&args[0], "first argument of 'UniV_addAuxWrites'", univ)?;
        univ.aux_stats.writes = univ.aux_stats.writes.saturating_add_signed(x);
        Ok(UniLValue::Int(univ.aux_stats.writes as i64))
    }

    UniV_addAuxReads(args, [UniLType::Int], univ, _task) -> (UniLType::Int) {
        let x = expect_int_strict(&args[0], "first argument of 'UniV_addAuxReads'", univ)?;
        univ.aux_stats.reads = univ.aux_stats.reads.saturating_add_signed(x);
        Ok(UniLValue::Int(univ.aux_stats.reads as i64))
    }

    UniV_addAuxSwaps(args, [UniLType::Int], univ, _task) -> (UniLType::Int) {
        let x = expect_int_strict(&args[0], "first argument of 'UniV_addAuxSwaps'", univ)?;
        univ.aux_stats.swaps = univ.aux_stats.swaps.saturating_add_signed(x);
        univ.aux_stats.writes = univ.aux_stats.writes.saturating_add_signed(x * 2);
        Ok(UniLValue::Int(univ.aux_stats.swaps as i64))
    }

    UniV_addAuxComparisons(args, [UniLType::Int], univ, _task) -> (UniLType::Int) {
        let x = expect_int_strict(&args[0], "first argument of 'UniV_addAuxComparisons'", univ)?;
        univ.comparisons = univ.comparisons.saturating_add_signed(x);
        univ.aux_stats.reads = univ.aux_stats.reads.saturating_add_signed(x * 2);
        Ok(UniLValue::Int(univ.comparisons as i64))
    }
}

pub fn color_type() -> UniLType {
    UniLType::Object { 
        fields: {
            let mut fields = HashMap::new();
            fields.insert(Rc::from("r"), UniLType::Int);
            fields.insert(Rc::from("g"), UniLType::Int);
            fields.insert(Rc::from("b"), UniLType::Int);
            Rc::new(RefCell::new(fields))
        } 
    }
}