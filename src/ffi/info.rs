#![allow(dead_code)]

use libc::{c_int};

static STRING: c_int   = 0x100000;
static LONG: c_int     = 0x200000;
static DOUBLE: c_int   = 0x300000;
static SLIST:  c_int   = 0x400000;
static MASK: c_int     = 0x0fffff;
static TYPEMASK: c_int = 0xf00000;

pub type Key = c_int;

macro_rules! DEFINFO(
  ($name:ident, $ty:ident, $num:expr) => (pub static $name: Key = $ty + $num;);
)

DEFINFO!(EFFECTIVE_URL, STRING, 1)
DEFINFO!(RESPONSE_CODE, LONG,   2)
DEFINFO!(TOTAL_TIME, DOUBLE,    5)
