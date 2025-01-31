use super::{AChildNode, Element};
use std::collections::HashMap;
use uuid::Uuid;
use wasm_bindgen::UnwrapThrowExt;

pub trait Keyed
where
    for<'k> ListItemKey: From<&'k <Self as Keyed>::Key>,
{
    type Key: PartialEq<ListItemKey>;
    fn key(&self) -> &Self::Key;
}

impl<T: Keyed> Keyed for &T
where
    ListItemKey: for<'k> From<&'k <T as Keyed>::Key>,
{
    type Key = T::Key;
    fn key(&self) -> &Self::Key {
        (*self).key()
    }
}

#[derive(PartialEq, Eq, Hash, Clone)]
pub enum ListItemKey {
    String(String),
    ISize(isize),
    USize(usize),
    I64(i64),
    U64(u64),
    I32(i32),
    U32(u32),
    Uuid(Uuid),
}

impl From<&String> for ListItemKey {
    fn from(value: &String) -> Self {
        ListItemKey::String(value.to_string())
    }
}

impl From<&&str> for ListItemKey {
    fn from(value: &&str) -> Self {
        ListItemKey::String(value.to_string())
    }
}

impl PartialEq<ListItemKey> for String {
    fn eq(&self, other: &ListItemKey) -> bool {
        match other {
            ListItemKey::String(value) => value == self,
            _ => false,
        }
    }
}

impl PartialEq<ListItemKey> for &str {
    fn eq(&self, other: &ListItemKey) -> bool {
        match other {
            ListItemKey::String(value) => value == self,
            _ => false,
        }
    }
}

macro_rules! impl_from_and_partial_eq_for_key_type {
    ($($key_type:ident $KeyVariant:ident)+) => {
        $(
            impl From<&$key_type> for ListItemKey {
                fn from(value: &$key_type) -> Self {
                    ListItemKey::$KeyVariant(*value)
                }
            }
            impl PartialEq<ListItemKey> for $key_type {
                fn eq(&self, other: &ListItemKey) -> bool {
                    match other {
                        ListItemKey::$KeyVariant(value) => value == self,
                        _ => false,
                    }
                }
            }
        )+
    };
}

impl_from_and_partial_eq_for_key_type! {
    isize ISize
    usize USize
    i64 I64
    u64 U64
    i32 I32
    u32 U32
    Uuid Uuid
}

#[derive(Debug)]
pub struct OldElement {
    pub index: usize,
    pub element: Element,
}

pub struct KeyedElement {
    pub key: ListItemKey,
    pub element: Element,
}

impl KeyedElement {
    pub fn new(key: ListItemKey, element: Element) -> Self {
        Self { key, element }
    }
}

pub struct ListItemTemplate {
    pub rendered: bool,
    pub element: Element,
}

#[derive(Default)]
pub struct KeyedList {
    active: Vec<Option<KeyedElement>>,
    // The primary reason for the double buffer here is for easy implementation.
    buffer: Vec<Option<KeyedElement>>,
    template: Option<ListItemTemplate>,
    old_elements_map: HashMap<ListItemKey, OldElement>,
}

impl Clone for KeyedList {
    fn clone(&self) -> Self {
        // No clone for keyed list
        // If cloning is applied to keyed-list, make sure that if the parent status
        // is ElementStatus::Cloned, then every element in the list should also
        // have status=ElementStatus::Cloned
        Self {
            active: Vec::new(),
            buffer: Vec::new(),
            old_elements_map: HashMap::new(),
            template: None,
        }
    }
}

impl KeyedList {
    #[cfg(test)]
    pub fn active_nodes(&self) -> &Vec<Option<KeyedElement>> {
        &self.active
    }

    pub fn get_first_element(&self) -> Option<&Element> {
        self.active
            .first()
            .and_then(|i| i.as_ref().map(|ke| &ke.element))
    }

    pub fn get_last_element(&self) -> Option<&Element> {
        self.active
            .last()
            .and_then(|i| i.as_ref().map(|ke| &ke.element))
    }

    pub fn require_init_template(&mut self, f: impl FnOnce() -> Element) -> bool {
        match self.template.as_mut() {
            None => {
                self.template = Some(ListItemTemplate {
                    rendered: false,
                    element: f(),
                });
                true
            }
            Some(t) => !t.rendered,
        }
    }

    #[allow(clippy::type_complexity)]
    pub fn items_mut(
        &mut self,
    ) -> (
        Option<&mut ListItemTemplate>,
        &mut Vec<Option<KeyedElement>>,
        &mut Vec<Option<KeyedElement>>,
        &mut HashMap<ListItemKey, OldElement>,
    ) {
        (
            self.template.as_mut(),
            &mut self.buffer,
            &mut self.active,
            &mut self.old_elements_map,
        )
    }

    // better name?
    pub fn pre_update(&mut self, count: usize) {
        self.old_elements_map.reserve(count);
        if count < self.buffer.len() {
            self.buffer.truncate(count);
        } else {
            self.buffer
                .extend((0..(count - self.buffer.len())).map(|_| None));
        }
        debug_assert_eq!(count, self.buffer.len());
        std::mem::swap(&mut self.active, &mut self.buffer);
    }

    pub fn remove_from_dom(self, parent: &web_sys::Node) {
        self.active.iter().for_each(|item| {
            item.as_ref()
                .expect_throw("dom::keyed_list::KeyedList::clear")
                .element
                .remove_from(parent)
        });
    }

    pub fn append_to(&self, parent: &web_sys::Node) {
        self.active.iter().for_each(|item| {
            item.as_ref()
                .expect_throw("dom::keyed_list::KeyedList::append_to")
                .element
                .append_to(parent)
        });
    }
}
