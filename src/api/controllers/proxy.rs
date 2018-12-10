use failure::Fail;
use futures::future;
use futures::prelude::*;

use super::super::utils::{parse_body, response_with_model};
use super::Context;
use super::ControllerFuture;
use api::error::*;

pub fn proxy(ctx: &Context) -> ControllerFuture {
    unimplemented!()
}
