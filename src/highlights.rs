use std::hash::Hash;
use raylib::color::Color;

use crate::{univm::object::{AnyObject, ExecutionInterrupt, List, Object, UniLValue}, utils::object::{expect_object, fetch_int_field, fetch_int_field_range_strict, fetch_optional_bool_field, fetch_optional_object_field}, UniV};

#[derive(Clone, Debug)]
pub struct HighlightInfo {
    pub idx:      usize,
    pub aux:      Option<*const AnyObject>,
    pub color:    Option<Color>,
    pub silent:   bool,
    pub is_write: bool
}

impl HighlightInfo {
    pub fn new(idx: usize, aux: Option<*const AnyObject>, color: Option<Color>, silent: bool, is_write: bool) -> Self {
        Self { idx, aux, color, silent, is_write }
    }

    pub fn from_idx(idx: usize) -> Self {
        Self { idx, aux: None, color: None, silent: false, is_write: false }
    }

    pub fn from_idx_and_aux(idx: usize, aux: Option<*const AnyObject>) -> Self {
        Self { idx, aux, color: None, silent: false, is_write: false }
    }

    pub fn from_idx_and_aux_write(idx: usize, aux: Option<*const AnyObject>) -> Self {
        Self { idx, aux, color: None, silent: false, is_write: true }
    }

    pub fn from_obj(value: &UniLValue, univ: &mut UniV) -> Result<Self, ExecutionInterrupt> {
        let obj = expect_object(value, "HighlightInfo", univ)?;
        if let AnyObject::AnonObject(instance) = &*obj.borrow() {
            let orig_idx = fetch_int_field(instance, "idx", "HighlightInfo")
                .map_err(|e| univ.vm.create_exception(UniLValue::String(e)))?;

            let idx;
            let aux = {
                match fetch_optional_object_field(instance, "aux") {
                    UniLValue::Null => {
                        idx = List::convert_index_for_len(univ.shared.array.len(), orig_idx);
                        None
                    }
                    UniLValue::Object(obj) => {
                        if let AnyObject::List(list) = &*obj.borrow() {
                            idx = list.convert_index(orig_idx);
                            univ.get_optional_aux_id(obj.as_ptr() as *const AnyObject)
                        } else {
                            return Err(univ.vm.create_exception(UniLValue::String(format!(
                                "Auxiliary array for highlight must be a list (got {})",
                                obj.borrow().stringify_type()
                            ).into())));
                        }
                    }
                    _ => unreachable!()
                }
            };

            let color = {
                if let UniLValue::Object(color_obj) = fetch_optional_object_field(instance, "color") {
                    if let AnyObject::AnonObject(color_instance) = &*color_obj.borrow() {
                        let r = fetch_int_field_range_strict(color_instance, 0..255, "r", "Color")
                            .map_err(|e| univ.vm.create_exception(UniLValue::String(e)))? as u8;
                        let g = fetch_int_field_range_strict(color_instance, 0..255, "g", "Color")
                            .map_err(|e| univ.vm.create_exception(UniLValue::String(e)))? as u8;
                        let b = fetch_int_field_range_strict(color_instance, 0..255, "b", "Color")
                            .map_err(|e| univ.vm.create_exception(UniLValue::String(e)))? as u8;

                        Some(Color { r, g, b, a: 255 })
                    } else {
                        return Err(univ.vm.create_exception(UniLValue::String(format!(
                            "Cannot form Color from {}",
                            color_obj.borrow().stringify_type()
                        ).into())));
                    }
                } else {
                    None
                }
            };

            let silent = fetch_optional_bool_field(instance, "silent", false);
            let is_write = fetch_optional_bool_field(instance, "isWrite", false);

            Ok(HighlightInfo { idx, aux, color, silent, is_write })
        } else {
            Err(univ.vm.create_exception(UniLValue::String(format!(
                "Cannot form HighlightInfo from {}",
                value.stringify_type()
            ).into())))
        }
    }
}

impl Eq for HighlightInfo {}
impl PartialEq for HighlightInfo {
    fn eq(&self, other: &Self) -> bool {
        self.idx == other.idx && (
            (
                self.aux.is_none() && other.aux.is_none()
            ) || (
                self.aux.is_some() && other.aux.is_some() &&
                self.aux.unwrap() == other.aux.unwrap()
            )
        )
    }
}

impl Hash for HighlightInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.idx.hash(state);

        if let Some(aux) = &self.aux {
            aux.hash(state);
        } else {
            0.hash(state);
        }
    }
}