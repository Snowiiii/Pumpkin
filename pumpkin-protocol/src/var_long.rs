use crate::{VarIntDecodeError, VarLongType};

use crate::var_int_helper::{self, impl_var_int, VarEncodedInteger};
use std::num::NonZero;

impl_var_int! {
    VarLong(VarLongType) {
        max = 10
        i64 => u64
    }
}

impl From<u32> for VarLong {
    fn from(value: u32) -> Self {
        Self(value as i64)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rayon::iter::{IntoParallelIterator, ParallelIterator};

    #[test]
    fn serde_works() {
        assert!((i64::MIN..=i64::MAX).into_par_iter().all(|i| {
            VarLong::new(i)
                .encode(|buff| i == VarLong::decode_from_slice(&mut &*buff).unwrap().get())
        }))
    }
}
