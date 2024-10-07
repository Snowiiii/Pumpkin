use std::{borrow::Cow, marker::PhantomData};

use heed::{BytesDecode, BytesEncode};
use speedy::{LittleEndian, Readable, Writable};

pub struct Speedy<'t, T>(PhantomData<&'t T>);

impl<'t, T: Writable<LittleEndian>> BytesEncode<'t> for Speedy<'t, T> {
    type EItem = T;

    fn bytes_encode(item: &'t Self::EItem) -> Result<Cow<'t, [u8]>, heed::BoxedError> {
        
        let bytes = item.write_to_vec()?;
        Ok(Cow::Owned(bytes))
    }
}

impl<'t, T: Readable<'t, LittleEndian>> BytesDecode<'t> for Speedy<'t, T> {
    type DItem = T;

    fn bytes_decode(bytes: &'t [u8]) -> Result<Self::DItem, heed::BoxedError> {
        
        Ok(T::read_from_buffer(bytes)?)
    }
}
