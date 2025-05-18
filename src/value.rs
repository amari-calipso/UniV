use crate::univm::object::AnyObject;

#[derive(Clone, Default, Eq)]
pub struct VerifyValue {
    pub value: i64,
    pub idx:   usize
}

impl VerifyValue {
    pub fn new(value: i64, idx: usize) -> Self {
        Self { value, idx }
    }
}

impl PartialEq for VerifyValue {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialOrd for VerifyValue {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

impl Ord for VerifyValue {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.value.cmp(&other.value)
    }
}

#[derive(Clone, Eq, Ord)]
pub struct Value {
    pub value: i64,
    pub idx:   usize,
    pub aux:   Option<*const AnyObject>
}

impl Value {
    pub fn new(value: i64, idx: usize, aux: Option<*const AnyObject>) -> Self {
        Value { value, idx, aux }
    }

    pub fn pos_value(&self) -> i64 {
        if self.value < 0 {
            0
        } else {
            self.value
        }
    }
}

impl PartialEq for Value {
    fn eq(&self, other: &Self) -> bool {
        self.value == other.value
    }
}

impl PartialOrd for Value {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.value.partial_cmp(&other.value)
    }
}