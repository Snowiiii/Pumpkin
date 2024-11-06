use std::num::NonZero;

use crate::{var_int_helper, VarEncodedInteger, VarIntType};
use crate::var_int_helper::{impl_var_int, VarIntDecodeError};

impl_var_int! {
    VarInt(VarIntType) {
        max = 5
        i32 => u32
    }
}

#[cfg(test)]
mod tests {
    use rayon::iter::{IntoParallelIterator, ParallelIterator};
    use super::*;
    
    #[test]
    #[ignore]
    fn serde_works() {
        assert!((i32::MIN..=i32::MAX).into_par_iter().all(|i| {
            VarInt::new(i).encode(|buff| {
                i == VarInt::decode_from_slice(&mut &*buff).unwrap().get()
            })
        }))
    }
}