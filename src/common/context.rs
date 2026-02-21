use std::rc::Rc;

use hashbrown::HashMap;

use crate::Value;

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
#[derive(Debug, PartialEq)]
pub(crate) struct Context {
    global: HashMap<Rc<str>, Option<Value>>,
    template: HashMap<u32, HashMap<Rc<str>, Option<Value>>>,
    type_: HashMap<Rc<str>, HashMap<Rc<str>, Option<Value>>>,
    user: HashMap<Rc<str>, HashMap<Rc<str>, Option<Value>>>,
}

impl Context {
    pub(crate) fn new() -> Self {
        Self {
            global: HashMap::new(),
            template: HashMap::new(),
            type_: HashMap::new(),
            user: HashMap::new(),
        }
    }

    pub(crate) fn reset(&mut self) {
        self.global.clear();
        self.template.clear();
        self.type_.clear();
        self.user.clear();
    }

    pub(crate) fn set(&mut self, dict: DictionaryType, key: Rc<str>, val: Option<Value>) {
        match dict {
            DictionaryType::Global => {
                self.global.insert(key, val);
            }
            DictionaryType::Template(id) => {
                if self.template.contains_key(&id) {
                    self.template.get_mut(&id).unwrap().insert(key, val);
                } else {
                    let mut hm = HashMap::new();
                    hm.insert(key, val);
                    self.template.insert(id, hm);
                }
            }
            DictionaryType::Type(name) => {
                if self.type_.contains_key(&name) {
                    self.type_.get_mut(&name).unwrap().insert(key, val);
                } else {
                    let mut hm = HashMap::new();
                    hm.insert(key, val);
                    self.type_.insert(name, hm);
                }
            }
            DictionaryType::UserDefined(name) => {
                if self.user.contains_key(&name) {
                    self.user.get_mut(&name).unwrap().insert(key, val);
                } else {
                    let mut hm = HashMap::new();
                    hm.insert(key, val);
                    self.user.insert(name, hm);
                }
            }
        }
    }

    pub(crate) fn get(&self, dict: DictionaryType, key: &Rc<str>) -> Option<Option<Value>> {
        match dict {
            DictionaryType::Global => self.global.get(key).cloned(),
            DictionaryType::Template(id) => match self.template.get(&id) {
                None => None,
                Some(hm) => hm.get(key).cloned(),
            },
            DictionaryType::Type(name) => match self.type_.get(&name) {
                None => None,
                Some(hm) => hm.get(key).cloned(),
            },
            DictionaryType::UserDefined(name) => match self.user.get(&name) {
                None => None,
                Some(hm) => hm.get(key).cloned(),
            },
        }
    }
}
