use std::{cell::RefCell, collections::HashMap, rc::Rc};

use super::type_system::UniLType;

pub enum AnalyzerEnvError {
    Unknown,
    Global
}

#[derive(Debug, Clone)]
pub struct AnalyzerEnvironment {
    values:        HashMap<Rc<str>, UniLType>,
    pub enclosing: Option<Rc<RefCell<AnalyzerEnvironment>>>
}

impl AnalyzerEnvironment {
    pub fn new() -> Self {
        AnalyzerEnvironment { values: HashMap::new(), enclosing: None }
    }

    pub fn with_enclosing(enclosing: Rc<RefCell<AnalyzerEnvironment>>) -> Self {
        AnalyzerEnvironment { values: HashMap::new(), enclosing: Some(enclosing) }
    }

    pub fn get_locals(&self) -> &HashMap<Rc<str>, UniLType> {
        &self.values
    }

    pub fn define(&mut self, name: &Rc<str>, value: UniLType) {
        if name.as_ref() == "_" {
            return;
        }

        self.values.insert(Rc::clone(name), value);
    }

    pub fn set(&mut self, name: &Rc<str>, value: UniLType) -> Result<(), AnalyzerEnvError> {
        if name.as_ref() == "_" {
            return Ok(());
        }

        if self.values.contains_key(name) {
            self.values.insert(Rc::clone(name), value);
            Ok(())
        } else if let Some(enclosing) = &self.enclosing {
            let mut env = enclosing.borrow_mut();
            if env.enclosing.is_some() {
                env.set(name, value)
            } else if env.values.contains_key(name) {
                Err(AnalyzerEnvError::Global)
            } else {
                Err(AnalyzerEnvError::Unknown)
            }
        } else {
            Err(AnalyzerEnvError::Unknown)
        }
    }

    pub fn set_global(&mut self, name: &Rc<str>, value: UniLType) -> Result<(), ()> {
        if name.as_ref() == "_" {
            return Ok(());
        }

        if self.values.contains_key(name) {
            self.values.insert(Rc::clone(name), value);
            Ok(())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().set_global(name, value)
        } else {
            Err(())
        }
    }

    pub fn del(&mut self, name: &str) -> Result<(), ()> {
        if self.values.contains_key(name) {
            self.values.remove(name);
            Ok(())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow_mut().del(name)
        } else {
            Err(())
        }
    }

    pub fn get(&self, name: &str) -> Option<UniLType> {
        if let Some(var) = self.values.get(name) {
            Some(var.clone())
        } else if let Some(enclosing) = &self.enclosing {
            enclosing.borrow().get(name)
        } else {
            None
        }
    }
}