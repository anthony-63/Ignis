use std::mem;

use crate::lexer::Token;

pub struct LookupTable<T> {
    values: Vec<(Token, T)>,
}

impl<T> LookupTable<T> {
    pub fn new() -> Self {
        Self {
            values: vec![],
        }
    }

    pub fn insert(&mut self, key: Token, value: T) {
        let v = self.get_mut(&key);
        if let Some(k) = v {
            *k = value;
        } else {
            self.values.push((key, value));
        }
    }

    pub fn get(&self, key: &Token) -> Option<&T> {
        for (k, v) in self.values.iter() {
            if mem::discriminant(key) == mem::discriminant(k) {
                return Some(v);
            }
        }

        None
    }
    
    pub fn get_mut(&mut self, key: &Token) -> Option<&mut T> {
        for (k, v) in self.values.iter_mut() {
            if mem::discriminant(key) == mem::discriminant(k) {
                return Some(v);
            }
        }

        None
    }
}
