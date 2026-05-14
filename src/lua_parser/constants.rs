use std::borrow::Cow;

#[allow(missing_docs)]
#[derive(Debug, Clone, PartialEq)]
pub enum LuaConstant<'input> {
    Nil,
    Boolean(bool),
    Number(f64),
    LuaString(Cow<'input, [u8]>),
}

impl<'input> LuaConstant<'input> {
    #[allow(missing_docs)]
    pub fn as_string(&self) -> Option<&[u8]> {
        match self {
            Self::LuaString(s) => Some(s.as_ref()),
            _ => None,
        }
    }
}
