use indexmap::IndexMap;
use super::value::LuaValue;

#[derive(Debug, Clone, Default)]
#[allow(missing_docs)]
pub struct LuaTable {
    pub array: Vec<LuaValue>,
    pub hash: IndexMap<LuaValue, LuaValue>,
}

impl PartialEq for LuaTable {
    fn eq(&self, other: &Self) -> bool {
        self.array == other.array && self.hash == other.hash
    }
}

impl LuaTable {
    #[allow(missing_docs)]
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(missing_docs)]
    pub fn get(&self, key: &LuaValue) -> LuaValue {
        match key {
            LuaValue::Int(i) if *i > 0 && (*i as usize) <= self.array.len() => {
                self.array[(*i as usize) - 1].clone()
            }
            LuaValue::Float(f) if f.fract() == 0.0 && *f > 0.0 && (*f as usize) <= self.array.len() => {
                self.array[(*f as usize) - 1].clone()
            }
            _ => self.hash.get(key).cloned().unwrap_or(LuaValue::Nil),
        }
    }

    #[allow(missing_docs)]
    pub fn set(&mut self, key: LuaValue, value: LuaValue) {
        match key {
            LuaValue::Int(i) if i > 0 => {
                let idx = (i as usize) - 1;
                if idx == self.array.len() {
                    self.array.push(value);
                } else if idx < self.array.len() {
                    self.array[idx] = value;
                } else {
                    self.hash.insert(key, value);
                }
            }
            LuaValue::Float(f) if f.fract() == 0.0 && f > 0.0 => {
                let idx = (f as usize) - 1;
                if idx == self.array.len() {
                    self.array.push(value);
                } else if idx < self.array.len() {
                    self.array[idx] = value;
                } else {
                    self.hash.insert(key, value);
                }
            }
            _ => {
                self.hash.insert(key, value);
            }
        }
    }
}
