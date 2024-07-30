use std::{collections::HashMap, fmt, io, ops::Index, string::FromUtf8Error};

#[derive(Debug)]
pub enum ParseError {
    InvalidType(u8),
    InvalidString(FromUtf8Error),
    IO(io::Error),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WrongTag(Tag);

impl fmt::Display for WrongTag {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "wrong tag: {:?}", self.0)
    }
}

impl std::error::Error for WrongTag {}

/// This is an nbt tag. It has a name, and any amount of data. This can be used
/// to store item data, entity data, level data, and more.
#[derive(Debug, Clone, PartialEq)]
pub struct NBT {
    pub tag: Tag,
    pub name: String,
}

impl Default for NBT {
    fn default() -> Self {
        NBT::new("", Tag::new_compound(&[]))
    }
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
    List(Vec<Tag>),     // All elements must be the same type, and un-named.
    Compound(Compound), // Types can be any kind, and are named. Order is not defined.
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

/// An NBT Compound tag. This is essentially a map, with some extra helper
/// functions.
#[derive(Debug, Clone, PartialEq)]
pub struct Compound {
    pub inner: HashMap<String, Tag>,
}

impl Compound {
    pub fn new() -> Self {
        Compound {
            inner: HashMap::new(),
        }
    }
    pub fn insert(&mut self, key: impl Into<String>, value: impl Into<Tag>) {
        self.inner.insert(key.into(), value.into());
    }
    pub fn get_or_create_compound(&mut self, key: impl Into<String>) -> &mut Compound {
        self.inner
            .entry(key.into())
            .or_insert_with(|| Tag::Compound(Compound::new()))
            .compound_mut()
            .unwrap()
    }

    pub fn contains_key(&self, key: impl AsRef<str>) -> bool {
        self.inner.contains_key(key.as_ref())
    }

    pub fn iter(&self) -> std::collections::hash_map::Iter<String, Tag> {
        self.inner.iter()
    }
    pub fn iter_mut(&mut self) -> std::collections::hash_map::IterMut<String, Tag> {
        self.inner.iter_mut()
    }
}
impl IntoIterator for Compound {
    type Item = (String, Tag);
    type IntoIter = std::collections::hash_map::IntoIter<String, Tag>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}
impl<'a> IntoIterator for &'a Compound {
    type Item = (&'a String, &'a Tag);
    type IntoIter = std::collections::hash_map::Iter<'a, String, Tag>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}
impl<'a> IntoIterator for &'a mut Compound {
    type Item = (&'a String, &'a mut Tag);
    type IntoIter = std::collections::hash_map::IterMut<'a, String, Tag>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
    }
}
impl From<HashMap<String, Tag>> for Compound {
    fn from(v: HashMap<String, Tag>) -> Self {
        Compound { inner: v }
    }
}

impl Index<&str> for Compound {
    type Output = Tag;
    fn index(&self, index: &str) -> &Tag {
        &self.inner[index]
    }
}

impl From<bool> for Tag {
    fn from(v: bool) -> Self {
        Tag::Byte(v as i8)
    }
}
impl From<&str> for Tag {
    fn from(s: &str) -> Self {
        Tag::String(s.into())
    }
}
impl From<String> for Tag {
    fn from(s: String) -> Self {
        Tag::String(s)
    }
}
impl From<HashMap<String, Tag>> for Tag {
    fn from(v: HashMap<String, Tag>) -> Self {
        Tag::Compound(Compound::from(v))
    }
}

impl<T> From<Vec<T>> for Tag
where
    Tag: From<T>,
{
    fn from(list: Vec<T>) -> Self {
        Tag::List(list.into_iter().map(|it| it.into()).collect())
    }
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
                        panic!("the given list contains multiple types: {inner:?}");
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
    pub const fn empty() -> Self {
        NBT {
            tag: Tag::End,
            name: String::new(),
        }
    }

    /// Appends the given element to the list. This will panic if self is not a
    /// list, or if tag does not match the type of the existing elements.
    pub fn list_add(&mut self, tag: Tag) {
        if let Tag::List(inner) = &mut self.tag {
            if let Some(v) = inner.get(0) {
                if tag.ty() != v.ty() {
                    panic!("cannot add different types to list. current: {inner:?}, new: {tag:?}");
                } else {
                    inner.push(tag);
                }
            } else {
                // No elements yet, so we add this no matter what type it is.
                inner.push(tag);
            }
        } else {
            panic!("called list_add on non-list type: {self:?}");
        }
    }

    /// Appends the given element to the compound. This will panic if self is not
    /// a compound tag.
    pub fn compound_add(&mut self, name: String, value: Tag) {
        if let Tag::Compound(inner) = &mut self.tag {
            inner.insert(name, value);
        } else {
            panic!("called compound_add on non-compound type: {self:?}");
        }
    }

    /// If this is a compound tag, this returns the inner data of the tag.
    /// Otherwise, this panics.
    pub fn compound(&self) -> Option<&Compound> {
        if let Tag::Compound(inner) = &self.tag {
            Some(inner)
        } else {
            None
        }
    }
    /// If this is a compound tag, this returns the inner data of the tag.
    /// Otherwise, this panics.
    pub fn compound_mut(&mut self) -> Option<&mut Compound> {
        if let Tag::Compound(inner) = &mut self.tag {
            Some(inner)
        } else {
            None
        }
    }

    pub fn tag(&self) -> &Tag {
        &self.tag
    }
    pub fn into_tag(self) -> Tag {
        self.tag
    }
}

macro_rules! getter {
  ( $(: $conv:tt)? $name:ident -> $variant:ident ( $ty:ty ) ) => {
    pub fn $name(&self) -> Result<$ty, WrongTag> {
      match self {
        Self::$variant(v) => Ok($($conv)? v),
        _ => Err(WrongTag(self.clone())),
      }
    }
  };
}

impl Tag {
    /// A simpler way to construct compound tags inline.
    pub fn new_compound(value: &[(&str, Tag)]) -> Self {
        let mut inner = HashMap::new();
        for (name, tag) in value {
            inner.insert(name.to_string(), tag.clone());
        }
        inner.into()
    }

    getter!(:*byte -> Byte(i8));
    getter!(:*short -> Short(i16));
    getter!(:*int -> Int(i32));
    getter!(:*long -> Long(i64));
    getter!(:*float -> Float(f32));
    getter!(:*double -> Double(f64));
    getter!(string -> String(&str));
    getter!(byte_arr -> ByteArr(&[u8]));
    getter!(list -> List(&Vec<Tag>));
    getter!(compound -> Compound(&Compound));
    getter!(long_arr -> LongArray(&Vec<i64>));

    pub fn compound_mut(&mut self) -> Result<&mut Compound, WrongTag> {
        match self {
            Self::Compound(v) => Ok(v),
            _ => Err(WrongTag(self.clone())),
        }
    }
}
