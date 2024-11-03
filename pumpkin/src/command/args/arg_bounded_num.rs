use core::f64;
use std::str::FromStr;

use async_trait::async_trait;

use crate::command::tree::RawArgs;
use crate::command::CommandSender;
use crate::server::Server;

use super::super::args::ArgumentConsumer;
use super::{Arg, DefaultNameArgConsumer};

/// Consumes a single generic num, but only if it's in bounds.
pub(crate) struct BoundedNumArgumentConsumer<T: ArgNum> {
    min_inclusive: Option<T>,
    max_inclusive: Option<T>,
    name: Option<&'static str>,
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
    pub(crate) const fn new() -> Self {
        Self {
            min_inclusive: None,
            max_inclusive: None,
            name: None,
        }
    }

    pub(crate) const fn min(mut self, min_inclusive: T) -> Self {
        self.min_inclusive = Some(min_inclusive);
        self
    }

    #[allow(unused)]
    pub(crate) const fn max(mut self, max_inclusive: T) -> Self {
        self.max_inclusive = Some(max_inclusive);
        self
    }

    pub(crate) const fn name(mut self, name: &'static str) -> Self {
        self.name = Some(name);
        self
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

impl<T: ArgNum> DefaultNameArgConsumer for BoundedNumArgumentConsumer<T> {
    fn default_name(&self) -> &'static str {
        // setting a single default name for all BoundedNumArgumentConsumer variants is probably a bad idea since it would lead to confusion
        self.name.expect("Only use *_default variants of methods with a BoundedNumArgumentConsumer that has a name.")
    }

    fn get_argument_consumer(&self) -> &dyn ArgumentConsumer {
        self
    }
}
