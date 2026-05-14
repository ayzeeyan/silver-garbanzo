use std::cell::RefCell;
use std::rc::Rc;
use std::fmt::Display;

use super::table::LuaTable;

#[allow(missing_docs)]
pub type LuaTableRef = Rc<RefCell<LuaTable>>;
#[allow(missing_docs)]
pub type ClosureRef = Rc<RefCell<()>>; // Placeholder since we won't fully emulate closures for strings

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq, Default)]
pub enum LuaValue {
    #[default]
    Nil,
    Bool(bool),
    Int(i64),
    Float(f64),
    LuaString(Vec<u8>),
    Table(LuaTableRef),
    Closure(ClosureRef),
}

impl Display for LuaValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Nil => write!(f, "nil"),
            Self::Bool(b) => write!(f, "{}", b),
            Self::Int(i) => write!(f, "{}", i),
            Self::Float(fl) => write!(f, "{}", fl),
            Self::LuaString(s) => write!(f, "{}", String::from_utf8_lossy(s)),
            Self::Table(_) => write!(f, "table"),
            Self::Closure(_) => write!(f, "function"),
        }
    }
}

impl std::hash::Hash for LuaValue {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            Self::Nil => 0.hash(state),
            Self::Bool(b) => {
                1.hash(state);
                b.hash(state);
            }
            Self::Int(i) => {
                2.hash(state);
                i.hash(state);
            }
            Self::Float(fl) => {
                3.hash(state);
                fl.to_bits().hash(state);
            }
            Self::LuaString(s) => {
                4.hash(state);
                s.hash(state);
            }
            Self::Table(t) => {
                5.hash(state);
                Rc::as_ptr(t).hash(state);
            }
            Self::Closure(c) => {
                6.hash(state);
                Rc::as_ptr(c).hash(state);
            }
        }
    }
}

impl Eq for LuaValue {}
