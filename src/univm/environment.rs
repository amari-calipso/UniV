use std::{cell::RefCell, collections::HashMap, rc::Rc};

use crate::compiler::environment::AnalyzerEnvironment;

use super::object::UniLValue;

#[derive(Debug, Clone)]
pub struct Environment {
    pub values:    HashMap<Rc<str>, UniLValue>,
    pub enclosing: Option<Rc<RefCell<Environment>>>
}

impl Environment {
    pub fn new() -> Self {
        Environment { values: HashMap::new(), enclosing: None }
    }

    pub fn with_enclosing(enclosing: Rc<RefCell<Environment>>) -> Self {
        Environment { values: HashMap::new(), enclosing: Some(enclosing) }
    }

    #[allow(dead_code)] // for debugging
    pub fn print_vars(&self, indent: usize, depth: usize) {
        for (var, value) in &self.values {
            println!("{}{} = {}", " ".repeat(indent), var, value.stringify());
        }
        
        if depth != 0 {
            if let Some(enclosing) = &self.enclosing {
                enclosing.borrow().print_vars(indent + 1, depth - 1);
            }
        }
    }

    pub fn len(&self) -> usize {
        self.values.len()
    }

    pub fn define(&mut self, name: &Rc<str>, value: UniLValue) {
        if name.as_ref() == "_" {
            return;
        }

        self.values.insert(Rc::clone(name), value);
    }

    pub fn set(&mut self, name: &Rc<str>, value: UniLValue) -> Result<(), ()> {
        if name.as_ref() == "_" {
            return Ok(());
        }

        if self.values.contains_key(name) {
            self.values.insert(Rc::clone(name), value);
            Ok(())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().set(name, value)
        } else {
            Err(())
        }
    }

    pub fn del(&mut self, name: &str) -> Result<(), ()> {
        if name == "_" {
            return Err(());
        }

        if self.values.contains_key(name) {
            self.values.remove(name);
            Ok(())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().del(name)
        } else {
            Err(())
        }
    }

    pub fn get(&self, name: &str) -> Option<UniLValue> {
        if let Some(var) = self.values.get(name) {
            Some(var.clone())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow().get(name)
        } else {
            None
        }
    }

    pub fn to_analyzer(&self) -> AnalyzerEnvironment {
        assert!(self.enclosing.is_none());

        let mut env = AnalyzerEnvironment::new();
        for (key, value) in &self.values {
            env.define(key, value.clone().get_type());
        }

        env
    }
}
