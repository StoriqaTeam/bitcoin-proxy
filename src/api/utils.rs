use failure::Fail;
use futures::prelude::*;
use serde::Deserialize;
use serde_json;

use super::error::*;

pub fn parse_body<T>(body: Vec<u8>) -> impl Future<Item = T, Error = Error> + Send
where
    T: for<'de> Deserialize<'de> + Send,
{
    String::from_utf8(body.clone())
        .map_err(ectx!(ErrorContext::RequestUTF8, ErrorKind::BadRequest => body))
        .into_future()
        .and_then(|string| serde_json::from_str::<T>(&string).map_err(ectx!(ErrorContext::RequestJson, ErrorKind::BadRequest => string)))
}
