//! Bindings to the "easy" libcurl API.
//!
//! This module contains some simple types like `Easy` and `List` which are just
//! wrappers around the corresponding libcurl types. There's also a few enums
//! scattered about for various options here and there.
//!
//! Most simple usage of libcurl will likely use the `Easy` structure here, and
//! you can find more docs about its usage on that struct.

mod form;
mod handle;
mod handler;
mod list;
mod windows;

pub use self::form::{Form, Part};
pub use self::handle::{Easy, Transfer};
pub use self::handler::{Auth, NetRc, ProxyType, SslOpt};
pub use self::handler::{Easy2, Handler};
pub use self::handler::{HttpVersion, IpResolve, SslVersion, TimeCondition};
pub use self::handler::{InfoType, ReadError, SeekResult, WriteError};
pub use self::list::{Iter, List};
