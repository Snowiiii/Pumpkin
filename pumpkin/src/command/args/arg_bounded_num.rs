use core::f64;
use std::str::FromStr;

use async_trait::async_trait;
use pumpkin_protocol::client::play::{CommandSuggestion, ProtoCmdArgParser};

use crate::command::dispatcher::CommandError;
use crate::command::tree::RawArgs;
use crate::command::CommandSender;
use crate::server::Server;

use super::super::args::ArgumentConsumer;
use super::{Arg, DefaultNameArgConsumer, FindArg, GetClientSideArgParser};

/// Consumes a single generic num, but only if it's in bounds.
pub(crate) struct BoundedNumArgumentConsumer<T: ToFromNumber> {
    min_inclusive: Option<T>,
    max_inclusive: Option<T>,
    name: Option<&'static str>,
}

#[async_trait]
impl<T: ToFromNumber> ArgumentConsumer for BoundedNumArgumentConsumer<T>
where
    Self: GetClientSideArgParser,
{
    async fn consume<'a>(
        &'a self,
        _src: &CommandSender<'a>,
        _server: &'a Server,
        args: &mut RawArgs<'a>,
    ) -> Option<Arg<'a>> {
        let x = args.pop()?.parse::<T>().ok()?;

        if let Some(max) = self.max_inclusive {
            if x > max {
                return Some(Arg::Num(Err(())));
            }
        }

        if let Some(min) = self.min_inclusive {
            if x < min {
                return Some(Arg::Num(Err(())));
            }
        }

        Some(Arg::Num(Ok(x.to_number())))
    }

    async fn suggest<'a>(
        &'a self,
        _sender: &CommandSender<'a>,
        _server: &'a Server,
        _input: &'a str,
    ) -> Result<Option<Vec<CommandSuggestion>>, CommandError> {
        Ok(None)
    }
}

impl<T: ToFromNumber> FindArg<'_> for BoundedNumArgumentConsumer<T> {
    type Data = Result<T, NotInBounds>;

    fn find_arg(args: &super::ConsumedArgs, name: &str) -> Result<Self::Data, CommandError> {
        let Some(Arg::Num(result)) = args.get(name) else {
            return Err(CommandError::InvalidConsumption(Some(name.to_string())));
        };

        let data: Self::Data = match result {
            Ok(num) => {
                if let Some(x) = T::from_number(num) {
                    Ok(x)
                } else {
                    return Err(CommandError::InvalidConsumption(Some(name.to_string())));
                }
            }
            Err(()) => Err(()),
        };

        Ok(data)
    }
}

pub type NotInBounds = ();

#[derive(Clone, Copy)]
pub enum Number {
    F64(f64),
    F32(f32),
    I32(i32),
    #[allow(unused)]
    I64(i64),
}

impl<T: ToFromNumber> BoundedNumArgumentConsumer<T> {
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

pub(crate) trait ToFromNumber: PartialOrd + Copy + Send + Sync + FromStr {
    fn to_number(self) -> Number;
    fn from_number(arg: &Number) -> Option<Self>;
}

impl ToFromNumber for f64 {
    fn to_number(self) -> Number {
        Number::F64(self)
    }

    fn from_number(arg: &Number) -> Option<Self> {
        match arg {
            Number::F64(x) => Some(*x),
            _ => None,
        }
    }
}

impl GetClientSideArgParser for BoundedNumArgumentConsumer<f64> {
    fn get_client_side_parser(&self) -> ProtoCmdArgParser {
        ProtoCmdArgParser::Double {
            min: self.min_inclusive,
            max: self.max_inclusive,
        }
    }

    fn get_client_side_suggestion_type_override(
        &self,
    ) -> Option<pumpkin_protocol::client::play::ProtoCmdArgSuggestionType> {
        None
    }
}

impl ToFromNumber for f32 {
    fn to_number(self) -> Number {
        Number::F32(self)
    }

    fn from_number(arg: &Number) -> Option<Self> {
        match arg {
            Number::F32(x) => Some(*x),
            _ => None,
        }
    }
}

impl GetClientSideArgParser for BoundedNumArgumentConsumer<f32> {
    fn get_client_side_parser(&self) -> ProtoCmdArgParser {
        ProtoCmdArgParser::Float {
            min: self.min_inclusive,
            max: self.max_inclusive,
        }
    }

    fn get_client_side_suggestion_type_override(
        &self,
    ) -> Option<pumpkin_protocol::client::play::ProtoCmdArgSuggestionType> {
        None
    }
}

impl ToFromNumber for i32 {
    fn to_number(self) -> Number {
        Number::I32(self)
    }

    fn from_number(arg: &Number) -> Option<Self> {
        match arg {
            Number::I32(x) => Some(*x),
            _ => None,
        }
    }
}

impl GetClientSideArgParser for BoundedNumArgumentConsumer<i32> {
    fn get_client_side_parser(&self) -> ProtoCmdArgParser {
        ProtoCmdArgParser::Integer {
            min: self.min_inclusive,
            max: self.max_inclusive,
        }
    }

    fn get_client_side_suggestion_type_override(
        &self,
    ) -> Option<pumpkin_protocol::client::play::ProtoCmdArgSuggestionType> {
        None
    }
}

impl ToFromNumber for i64 {
    fn to_number(self) -> Number {
        Number::I64(self)
    }

    fn from_number(arg: &Number) -> Option<Self> {
        match arg {
            Number::I64(x) => Some(*x),
            _ => None,
        }
    }
}

impl GetClientSideArgParser for BoundedNumArgumentConsumer<i64> {
    fn get_client_side_parser(&self) -> ProtoCmdArgParser {
        ProtoCmdArgParser::Long {
            min: self.min_inclusive,
            max: self.max_inclusive,
        }
    }

    fn get_client_side_suggestion_type_override(
        &self,
    ) -> Option<pumpkin_protocol::client::play::ProtoCmdArgSuggestionType> {
        None
    }
}

impl<T: ToFromNumber> DefaultNameArgConsumer for BoundedNumArgumentConsumer<T>
where
    Self: ArgumentConsumer,
{
    fn default_name(&self) -> String {
        // setting a single default name for all BoundedNumArgumentConsumer variants is probably a bad idea since it would lead to confusion
        self.name.expect("Only use *_default variants of methods with a BoundedNumArgumentConsumer that has a name.").to_string()
    }
}
