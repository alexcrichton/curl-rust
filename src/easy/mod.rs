//! Bindings to the "easy" libcurl API.
//!
//! This module contains some simple types like `Easy` and `List` which are just
//! wrappers around the corresponding libcurl types. There's also a few enums
//! scattered about for various options here and there.
//!
//! Most simple usage of libcurl will likely use the `Easy` structure here, and
//! you can find more docs about its usage on that struct.

mod list;
mod form;
mod handle;

pub use self::list::{List, Iter};
pub use self::form::{Form, Part};
pub use self::handle::{Easy, Transfer, ProxyType, TimeCondition, IpResolve};
pub use self::handle::{HttpVersion, SslVersion, InfoType, ReadError, WriteError};
pub use self::handle::{NetRc, Auth, SslOpt};
