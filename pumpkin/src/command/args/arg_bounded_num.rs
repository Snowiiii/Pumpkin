use core::f64;
use std::str::FromStr;

use async_trait::async_trait;

use crate::command::tree::RawArgs;
use crate::command::CommandSender;
use crate::server::Server;

use super::super::args::ArgumentConsumer;
use super::Arg;

/// Consumes a single generic num, but only if it's in bounds.
pub(crate) struct BoundedNumArgumentConsumer<T: ArgNum> {
    min_inclusive: Option<T>,
    max_inclusive: Option<T>,
}

#[async_trait]
impl<T: ArgNum> ArgumentConsumer for BoundedNumArgumentConsumer<T> {
    async fn consume<'a>(
        &self,
        _src: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let x = args.pop()?.parse::<T>().ok()?;

        if let Some(max) = self.max_inclusive {
            if x > max {
                return None;
            }
        }

        if let Some(min) = self.min_inclusive {
            if x < min {
                return None;
            }
        }

        Some(x.to_arg())
    }
}

impl<T: ArgNum> BoundedNumArgumentConsumer<T> {
    #[allow(unused)]
    pub(crate) const fn unbounded() -> Self {
        Self {
            min_inclusive: None,
            max_inclusive: None,
        }
    }

    pub(crate) const fn min(min_inclusive: T) -> Self {
        Self {
            min_inclusive: Some(min_inclusive),
            max_inclusive: None,
        }
    }

    #[allow(unused)]
    pub(crate) const fn max(max_inclusive: T) -> Self {
        Self {
            min_inclusive: None,
            max_inclusive: Some(max_inclusive),
        }
    }

    #[allow(unused)]
    pub(crate) const fn minmax(min_inclusive: T, max_inclusive: T) -> Self {
        Self {
            min_inclusive: Some(min_inclusive),
            max_inclusive: Some(max_inclusive),
        }
    }
}

pub(crate) trait ArgNum: PartialOrd + Copy + Send + Sync + FromStr {
    fn to_arg<'a>(self) -> Arg<'a>;
}

impl ArgNum for f64 {
    fn to_arg<'a>(self) -> Arg<'a> {
        Arg::F64(self)
    }
}

impl ArgNum for f32 {
    fn to_arg<'a>(self) -> Arg<'a> {
        Arg::F32(self)
    }
}

impl ArgNum for i32 {
    fn to_arg<'a>(self) -> Arg<'a> {
        Arg::I32(self)
    }
}

impl ArgNum for u32 {
    fn to_arg<'a>(self) -> Arg<'a> {
        Arg::U32(self)
    }
}
