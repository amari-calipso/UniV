use std::{cell::RefCell, collections::{HashMap, HashSet}, hash::Hash, rc::Rc};

#[derive(Debug, Clone, Eq)]
pub enum UniLType {
    Any, Null,
    Int, Float, Value,
    String, 
    List, 
    
    Type(Box<UniLType>),
    Group(HashSet<UniLType>),

    Object {
        fields: Rc<RefCell<HashMap<Rc<str>, UniLType>>>
    },

    Callable {
        args: Vec<UniLType>,
        return_type: Box<UniLType>
    }
}

impl UniLType {
    pub fn finalize(self) -> UniLType {
        match self {
            UniLType::Null => UniLType::Any,
            _ => self
        }
    }

    pub fn stringify(&self) -> Rc<str> {
        match self {
            UniLType::Any    => Rc::from("any"),
            UniLType::Null   => Rc::from("Null"),
            UniLType::Int    => Rc::from("Int"),
            UniLType::Float  => Rc::from("Float"),
            UniLType::Value  => Rc::from("Value"),
            UniLType::String => Rc::from("String"),
            UniLType::List   => Rc::from("List"),
            UniLType::Object { .. } => Rc::from("Object"),
            UniLType::Type(x) => format!("type \"{}\"", x.stringify()).into(),
            UniLType::Callable { args, return_type } => {
                let mut output = String::from("Callable (");

                for (i, arg) in args.iter().enumerate() {
                    output.push_str(&arg.stringify());

                    if i != args.len() - 1 {
                        output.push_str(", ")
                    }
                }

                output.push_str(") ");
                output.push_str(&return_type.stringify());
                output.into()
            }
            UniLType::Group(x) => {
                let mut output = String::new();
                
                for (i, type_) in x.iter().enumerate() {
                    output.push_str(&type_.stringify());

                    if i != x.len() - 1 {
                        output.push_str(" | ")
                    }
                }

                output.into()
            }
        }
    }

    pub fn equals(&self, other: &UniLType) -> bool {
        match self {
            UniLType::Any    => !matches!(other, UniLType::Type(_)),
            UniLType::Null   => matches!(other, UniLType::Null   | UniLType::Any),
            UniLType::Int    => matches!(other, UniLType::Int    | UniLType::Any),
            UniLType::Float  => matches!(other, UniLType::Float  | UniLType::Any),
            UniLType::Value  => matches!(other, UniLType::Value  | UniLType::Any),
            UniLType::String => matches!(other, UniLType::String | UniLType::Any),
            UniLType::List   => matches!(other, UniLType::List   | UniLType::Any),
            UniLType::Object { .. } => matches!(other, UniLType::Object { .. } | UniLType::Any),
            UniLType::Type(x) => {
                if let UniLType::Type(y) = other {
                    x.equals(y)
                } else {
                    false
                }
            }
            UniLType::Group(x) => {
                if let UniLType::Group(y) = other {
                    for type_ in y {
                        if !x.contains(type_) {
                            return false;
                        }
                    }

                    true
                } else {
                    matches!(other, UniLType::Any) || x.contains(other)
                }
            }
            UniLType::Callable { args: x_args, return_type: x_ret } => {
                if let UniLType::Callable { args: y_args, return_type: y_ret } = other {
                    if x_args.len() != y_args.len() || !x_ret.equals(&y_ret) {
                        return false;
                    } 

                    for i in 0 .. x_args.len() {
                        if !x_args[i].equals(&y_args[i]) {
                            return false;
                        }
                    }

                    true
                } else {
                    matches!(other, UniLType::Any)
                }
            }
        }
    }

    pub fn make_group(&self, other: UniLType) -> UniLType {
        if self.equals(&other) {
            return other;
        }

        if UniLType::Any.equals(self) || UniLType::Any.equals(&other) {
            return UniLType::Any;
        }

        if let UniLType::Type(x) = self {
            if let UniLType::Type(y) = &other {
                return UniLType::Type(Box::new(x.make_group(*y.clone())));
            } else {
                return UniLType::Any;
            }
        }

        if let UniLType::Group(group) = self {
            if let UniLType::Group(mut other_group) = other {
                for type_ in group {
                    if !other_group.contains(type_) {
                        other_group.insert(type_.clone());
                    }
                }

                UniLType::Group(other_group)
            } else {
                let mut group = group.clone();
                if !group.contains(&other) {
                    group.insert(other);
                }

                UniLType::Group(group)
            }
        } else if let UniLType::Group(mut group) = other {
            if !group.contains(self) {
                group.insert(self.clone());
            }

            UniLType::Group(group)
        } else {
            UniLType::Group(HashSet::from([self.clone(), other]))
        }
    }
}

impl PartialEq for UniLType {
    fn eq(&self, other: &Self) -> bool {
        self.equals(other)
    }
}

impl Hash for UniLType {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        match self {
            UniLType::Type(x) => x.hash(state),
            UniLType::Group(x) => {
                for type_ in x {
                    type_.hash(state);
                }
            }
            UniLType::Callable { args, return_type } => {
                return_type.hash(state);
                for arg in args {
                    arg.hash(state);
                }
            }
            _ => ()
        }
    }
}