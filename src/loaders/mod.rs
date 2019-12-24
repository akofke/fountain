use std::collections::HashMap;
use std::borrow::Cow;

enum ParamVal {
    Int(i32),

}


pub struct ParamSet {
    params: HashMap<String, ParamVal>,

}

impl ParamSet {
    pub fn get_one<T>(&mut self) -> Option<T> {
    }
}