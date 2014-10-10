#![allow(dead_code)]

use libc::{c_int};

const STRING: c_int   = 0x100000;
const LONG: c_int     = 0x200000;
const DOUBLE: c_int   = 0x300000;
const SLIST:  c_int   = 0x400000;
const MASK: c_int     = 0x0fffff;
const TYPEMASK: c_int = 0xf00000;

pub type Key = c_int;

macro_rules! DEFINFO(
    ($name:ident, $ty:ident, $num:expr) => (pub const $name: Key = $ty + $num;);
)

DEFINFO!(EFFECTIVE_URL, STRING, 1)
DEFINFO!(RESPONSE_CODE, LONG,   2)
DEFINFO!(TOTAL_TIME, DOUBLE,    5)
