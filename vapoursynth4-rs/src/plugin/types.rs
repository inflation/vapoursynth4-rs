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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::key;

    #[test]
    fn builds_plugin_argument_types() {
        let mut builder = TypeBuilder::default();
        for (key, ty, name) in [
            (key!(c"a"), Type::Int, "int"),
            (key!(c"b"), Type::Float, "float"),
            (key!(c"c"), Type::Data, "data"),
            (key!(c"d"), Type::ANode, "anode"),
            (key!(c"e"), Type::VNode, "vnode"),
            (key!(c"f"), Type::AFrame, "aframe"),
            (key!(c"g"), Type::VFrame, "vframe"),
            (key!(c"h"), Type::Func, "func"),
        ] {
            assert_eq!(ty.to_args(), name);
            builder.add(key, ty, false);
        }
        let values = Key::from(key!(c"values"));
        builder.add(&values, Type::Array(Box::new(Type::Int)), true);
        let args = builder.build();
        assert_eq!(args.len(), 9);
        assert_eq!(args[&values].0.to_args(), "int[]");
        assert!(args[&values].1);
    }
}
