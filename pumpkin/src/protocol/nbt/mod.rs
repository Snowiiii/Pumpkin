use std::collections::HashMap;

mod deserialize;
mod serialize;

#[derive(Debug, Clone, PartialEq)]
pub struct NBT {
    tag: Tag,
    name: String,
}

/// This is a single tag. It does not contain a name, but has the actual data
/// for any of the nbt tags.
#[derive(Debug, Clone, PartialEq)]
pub enum Tag {
    End,
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArr(Vec<u8>),
    String(String),
    List(Vec<Tag>), // All elements must be the same type, and un-named.
    Compound(HashMap<String, Tag>), // Types can be any kind, and are named. Order is not defined.
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

impl NBT {
    /// Creates a new nbt tag. The tag value can be anything.
    ///
    /// # Panics
    /// This will panic if the tag is a list, and the values within that list
    /// contain multiple types. This is a limitation with the nbt data format:
    /// lists can only contain one type of data.
    pub fn new(name: &str, tag: Tag) -> Self {
        if let Tag::List(inner) = &tag {
            if let Some(v) = inner.get(0) {
                let ty = v.ty();
                for v in inner {
                    if v.ty() != ty {
                        panic!("the given list contains multiple types: {:?}", inner);
                    }
                }
            }
        }
        NBT {
            tag,
            name: name.into(),
        }
    }

    /// Creates an empty nbt tag.
    pub fn empty(name: &str) -> Self {
        NBT {
            tag: Tag::End,
            name: name.into(),
        }
    }

    /// Appends the given element to the list. This will panic if self is not a
    /// list, or if tag does not match the type of the existing elements.
    pub fn list_add(&mut self, tag: Tag) {
        if let Tag::List(inner) = &mut self.tag {
            if let Some(v) = inner.get(0) {
                if tag.ty() != v.ty() {
                    panic!(
                        "cannot add different types to list. current: {:?}, new: {:?}",
                        inner, tag
                    );
                } else {
                    inner.push(tag);
                }
            } else {
                // No elements yet, so we add this no matter what type it is.
                inner.push(tag);
            }
        } else {
            panic!("called list_add on non-list type: {:?}", self);
        }
    }

    /// Appends the given element to the compound. This will panic if self is not
    /// a compound tag.
    pub fn compound_add(&mut self, name: String, value: Tag) {
        if let Tag::Compound(inner) = &mut self.tag {
            inner.insert(name, value);
        } else {
            panic!("called compound_add on non-compound type: {:?}", self);
        }
    }

    /// If this is a compound tag, this returns the inner data of the tag.
    /// Otherwise, this panics.
    pub fn compound(&self) -> &HashMap<String, Tag> {
        if let Tag::Compound(inner) = &self.tag {
            &inner
        } else {
            panic!("called compound on non-compound type: {:?}", self);
        }
    }
}

impl Tag {
    /// A simpler way to construct compound tags inline.
    pub fn compound(value: &[(&str, Tag)]) -> Self {
        let mut inner = HashMap::new();
        for (name, tag) in value {
            inner.insert(name.to_string(), tag.clone());
        }
        Self::Compound(inner)
    }
}
