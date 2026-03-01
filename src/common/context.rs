use std::rc::Rc;

use ahash::HashMap;

use crate::Value;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum DictionaryType {
    Global,
    Template(u32),
    Type(Rc<str>),
    UserDefined(Rc<str>),
}

/// Decoder state that stores global state during all messages decoding.
/// Created when decoder is created.
/// Destroyed when decoder is destroyed.
/// Can be reset during messages decoding.
#[derive(Debug, PartialEq, Default)]
pub(crate) struct Context {
    values: HashMap<(DictionaryType, ValueKey), Option<Value>>,
}

type ValueKey = Rc<str>;

impl Context {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn reset(&mut self) {
        self.values.clear();
    }

    pub(crate) fn set(&mut self, dict: DictionaryType, key: ValueKey, val: Option<Value>) {
        self.values.insert((dict, key), val);
    }

    pub(crate) fn get(&self, dict: DictionaryType, key: &ValueKey) -> Option<Option<Value>> {
        self.values.get(&(dict, key.clone())).cloned()
    }
}

#[cfg(test)]
mod tests {
    use std::rc::Rc;

    use crate::{
        Value,
        common::context::{Context, DictionaryType},
    };

    #[test]
    fn global_set_get_some_value() {
        let mut context = Context::new();
        let value = Some(Value::Int32(1));

        let key = Rc::from("aboba");
        let dict = DictionaryType::Global;

        let before_set = context.get(dict.clone(), &key);
        assert!(
            before_set.is_none(),
            "Value is set before insertion: {before_set:#?}"
        );

        context.set(dict.clone(), key.clone(), value.clone());

        let after_set = context.get(dict, &key);
        assert_eq!(
            after_set,
            Some(value.clone()),
            "Values does not match after set: lhs ({after_set:?}) != rhs ({value:?})"
        );
    }

    #[test]
    fn global_set_get_none() {
        let mut context = Context::new();
        let value = None;

        let key = Rc::from("aboba");
        let dict = DictionaryType::Global;

        let before_set = context.get(dict.clone(), &key);
        assert!(
            before_set.is_none(),
            "Value is set before insertion: {before_set:#?}"
        );

        context.set(dict.clone(), key.clone(), value.clone());

        let after_set = context.get(dict, &key);
        assert_eq!(
            after_set,
            Some(value.clone()),
            "Values does not match after set: lhs ({after_set:?}) != rhs ({value:?})"
        );
    }

    #[test]
    fn template_set_get_some_value() {
        let mut context = Context::new();
        let value = Some(Value::Int32(1));

        let key = Rc::from("aboba");
        let dict = DictionaryType::Template(2);

        let before_set = context.get(dict.clone(), &key);
        assert!(
            before_set.is_none(),
            "Value is set before insertion: {before_set:#?}"
        );

        context.set(dict.clone(), key.clone(), value.clone());

        let after_set = context.get(dict, &key);
        assert_eq!(
            after_set,
            Some(value.clone()),
            "Values does not match after set: lhs ({after_set:?}) != rhs ({value:?})"
        );
    }

    #[test]
    fn template_set_get_none() {
        let mut context = Context::new();
        let value = None;

        let key = Rc::from("aboba");
        let dict = DictionaryType::Template(2);

        let before_set = context.get(dict.clone(), &key);
        assert!(
            before_set.is_none(),
            "Value is set before insertion: {before_set:#?}"
        );

        context.set(dict.clone(), key.clone(), value.clone());

        let after_set = context.get(dict, &key);
        assert_eq!(
            after_set,
            Some(value.clone()),
            "Values does not match after set: lhs ({after_set:?}) != rhs ({value:?})"
        );
    }

    #[test]
    fn type_set_get_some_value() {
        let mut context = Context::new();
        let value = Some(Value::Int32(1));

        let key = Rc::from("aboba");
        let dict = DictionaryType::Type(Rc::from("type"));

        let before_set = context.get(dict.clone(), &key);
        assert!(
            before_set.is_none(),
            "Value is set before insertion: {before_set:#?}"
        );

        context.set(dict.clone(), key.clone(), value.clone());

        let after_set = context.get(dict, &key);
        assert_eq!(
            after_set,
            Some(value.clone()),
            "Values does not match after set: lhs ({after_set:?}) != rhs ({value:?})"
        );
    }

    #[test]
    fn type_set_get_none() {
        let mut context = Context::new();
        let value = None;

        let key = Rc::from("aboba");
        let dict = DictionaryType::Type(Rc::from("type"));

        let before_set = context.get(dict.clone(), &key);
        assert!(
            before_set.is_none(),
            "Value is set before insertion: {before_set:#?}"
        );

        context.set(dict.clone(), key.clone(), value.clone());

        let after_set = context.get(dict, &key);
        assert_eq!(
            after_set,
            Some(value.clone()),
            "Values does not match after set: lhs ({after_set:?}) != rhs ({value:?})"
        );
    }

    #[test]
    fn user_set_get_some_value() {
        let mut context = Context::new();
        let value = Some(Value::Int32(1));

        let key = Rc::from("aboba");
        let dict = DictionaryType::UserDefined(Rc::from("type"));

        let before_set = context.get(dict.clone(), &key);
        assert!(
            before_set.is_none(),
            "Value is set before insertion: {before_set:#?}"
        );

        context.set(dict.clone(), key.clone(), value.clone());

        let after_set = context.get(dict, &key);
        assert_eq!(
            after_set,
            Some(value.clone()),
            "Values does not match after set: lhs ({after_set:?}) != rhs ({value:?})"
        );
    }

    #[test]
    fn user_set_get_none() {
        let mut context = Context::new();
        let value = None;

        let key = Rc::from("aboba");
        let dict = DictionaryType::UserDefined(Rc::from("type"));

        let before_set = context.get(dict.clone(), &key);
        assert!(
            before_set.is_none(),
            "Value is set before insertion: {before_set:#?}"
        );

        context.set(dict.clone(), key.clone(), value.clone());

        let after_set = context.get(dict, &key);
        assert_eq!(
            after_set,
            Some(value.clone()),
            "Values does not match after set: lhs ({after_set:?}) != rhs ({value:?})"
        );
    }
}
