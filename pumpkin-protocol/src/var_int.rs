use std::num::NonZero;

use crate::var_int_helper::{impl_var_int, VarIntDecodeError};
use crate::{var_int_helper, VarEncodedInteger, VarIntType};

impl_var_int! {
    VarInt(VarIntType) {
        max = 5
        i32 => u32
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rayon::iter::{IntoParallelIterator, ParallelIterator};

    #[test]
    #[ignore]
    fn serde_works() {
        assert!((i32::MIN..=i32::MAX).into_par_iter().all(|i| {
            VarInt::new(i).encode(|buff| i == VarInt::decode_from_slice(&mut &*buff).unwrap().get())
        }))
    }
}
