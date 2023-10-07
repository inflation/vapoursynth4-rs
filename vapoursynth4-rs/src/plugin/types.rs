use std::{borrow::Cow, collections::HashMap};

use crate::map::{Key, KeyStr};

pub enum Type {
    Int,
    Float,
    Data,
    ANode,
    VNode,
    AFrame,
    VFrame,
    Func,
    Array(Box<Type>),
}

impl Type {
    #[must_use]
    pub fn to_args(&self) -> Cow<'static, str> {
        use Type as t;
        match self {
            t::Int => "int".into(),
            t::Float => "float".into(),
            t::Data => "data".into(),
            t::ANode => "anode".into(),
            t::VNode => "vnode".into(),
            t::AFrame => "aframe".into(),
            t::VFrame => "vframe".into(),
            t::Func => "func".into(),
            t::Array(t) => t.to_args() + "[]",
        }
    }
}

pub struct TypeBuilder {
    types: HashMap<Key, (Type, bool)>,
}

impl TypeBuilder {
    #[must_use]
    pub fn new() -> Self {
        Self {
            types: HashMap::new(),
        }
    }

    pub fn add(&mut self, key: &KeyStr, ty: Type, opt: bool) -> &mut Self {
        self.types.insert(key.into(), (ty, opt));
        self
    }

    #[must_use]
    pub fn build(self) -> HashMap<Key, (Type, bool)> {
        self.types
    }
}

impl Default for TypeBuilder {
    fn default() -> Self {
        Self::new()
    }
}
