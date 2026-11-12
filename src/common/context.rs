use rustc_hash::FxHashMap as HashMap;

use crate::Value;
use crate::base::pmap::PresenceMap;
use crate::utils::stacked::Stacked;

/// Location of an instruction's previous value in the dictionaries.
/// It is resolved with templates are loaded, so no strings are hashed during processing.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum DictionaryType {
    /// The instruction doesn't use dictionaries (not resolved).
    None,
    /// Index of the entry in `global` or user-defined dictionary.
    Static(u32),
    /// Key index in the dictionary of the current template.
    Template(u32),
    /// Key index in the dictionary of the current application type.
    Type(u32),
}

/// The state of the previous value:
///     - `None` - undefined,
///     - `Sone(None)` - empty,
///     - `Some(Some(None))` - value assigned.
pub(crate) type Entry = Option<Option<Value>>;

/// Processing scopes of one message: the current template, application type and presence map.
/// They are kept by the decoder/encoder between messages, so the stacks are allocated once.
#[derive(Debug)]
pub(crate) struct Scopes {
    pub(crate) template: Stacked<u32>,
    pub(crate) type_ref: Stacked<u32>,
    pub(crate) presence_map: Stacked<PresenceMap>,
}

impl Scopes {
    pub(crate) fn new() -> Self {
        Self {
            template: Stacked::new_empty(),
            type_ref: Stacked::new(0),
            presence_map: Stacked::new_empty(),
        }
    }

    /// Resets the scopes for a new message; previous message processing could fail half way.
    pub(crate) fn reset(&mut self) {
        self.template.reset(None);
        self.type_ref.reset(Some(0));
        self.presence_map.reset(None);
    }
}

/// Decoder state that stores global state during all messages decoding.
/// Created when decoder is created.
/// Destroyed when decoder is destroyed.
/// Can be reset during messages decoding.
#[derive(Debug, PartialEq, Default)]
pub(crate) struct Context {
    // Entries of `global` and user-defined dictionaries indexed by `DictionaryType::Static`.
    statics: Vec<Entry>,
    // Entries of `template` dictionaries keyed by (template index, key index)
    templates: HashMap<(u32, u32), Entry>,
    // Entries of `type` dictionaries keyed by (template index, type index)
    types: HashMap<(u32, u32), Entry>,
}

impl Context {
    pub(crate) fn new() -> Self {
        Self::default()
    }

    pub(crate) fn reset(&mut self) {
        self.statics.clear();
        self.templates.clear();
        self.types.clear();
    }

    #[inline]
    pub(crate) fn set(&mut self, dict: DictionaryType, template: u32, type_index: u32, val: Option<Value>) {
        match dict {
            DictionaryType::Static(i) => {
                let i = i as usize;
                if i >= self.statics.len() {
                    self.statics.resize(i + 1, None);
                }
                self.statics[i] = Some(val);
            }
            DictionaryType::Template(i) => {
                self.templates.insert((template, i), Some(val));
            }
            DictionaryType::Type(i) => {
                self.types.insert((type_index, i), Some(val));
            }
            DictionaryType::None => unreachable!("dictionary is not resolved"),
        }
    }

    #[inline]
    pub(crate) fn get(&self, dict: DictionaryType, template: u32, type_index: u32) -> Option<&Option<Value>> {
        match dict {
            DictionaryType::Static(i) => self.statics.get(i as usize)?.as_ref(),
            DictionaryType::Template(i) => self.templates.get(&(template, i))?.as_ref(),
            DictionaryType::Type(i) => self.types.get(&(type_index, i))?.as_ref(),
            DictionaryType::None => unreachable!("dictionary is not resolved"),
        }
    }

    #[inline]
    pub(crate) fn get_mut(&mut self, dict: DictionaryType, template: u32, type_index: u32) -> Option<&mut Option<Value>> {
        match dict {
            DictionaryType::Static(i) => self.statics.get_mut(i as usize)?.as_mut(),
            DictionaryType::Template(i) => self.templates.get_mut(&(template, i))?.as_mut(),
            DictionaryType::Type(i) => self.types.get_mut(&(type_index, i))?.as_mut(),
            DictionaryType::None => unreachable!("dictionary is not resolved"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn set_get() {
        for slot in [
            DictionaryType::Static(3),
            DictionaryType::Template(1),
            DictionaryType::Type(2),
        ] {
            let mut context = Context::new();
            assert_eq!(context.get(slot, 1, 1), None, "{slot:?}");

            context.set(slot, 1, 1, Some(Value::Int32(1)));
            assert_eq!(context.get(slot, 1, 1), Some(&Some(Value::Int32(1))), "{slot:?}");

            context.set(slot, 1, 1, None);
            assert_eq!(context.get(slot, 1, 1), Some(&None), "{slot:?}");

            context.reset();
            assert_eq!(context.get(slot, 1, 1), None, "{slot:?}");
        }
    }

    #[test]
    fn scoped_dicts_are_separate() {
        let mut context = Context::new();
        context.set(DictionaryType::Template(0), 1, 0, Some(Value::Int32(1)));
        context.set(DictionaryType::Type(0), 0, 1, Some(Value::Int32(2)));
        context.set(DictionaryType::Static(0), 1, 1, Some(Value::Int32(3)));

        assert_eq!(context.get(DictionaryType::Template(0), 2, 0), None);
        assert_eq!(context.get(DictionaryType::Type(0), 0, 2), None);

        assert_eq!(context.get(DictionaryType::Template(0), 1, 1), Some(&Some(Value::Int32(1))));
        assert_eq!(context.get(DictionaryType::Type(0), 1, 1), Some(&Some(Value::Int32(2))));
        assert_eq!(context.get(DictionaryType::Static(0), 2, 2), Some(&Some(Value::Int32(3))));
    }
}
