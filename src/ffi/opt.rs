use std::mem;
use libc::{c_void,c_int};

static LONG: c_int          = 0;
static OBJECTPOINT: c_int   = 10_000;
static FUNCTIONPOINT: c_int = 20_000;
static OFF_T: c_int         = 30_000;

pub type Opt = c_int;

pub trait OptVal {
  fn to_c_repr(self) -> *c_void;
}

impl OptVal for int {
  fn to_c_repr(self) -> *c_void {
    unsafe { mem::transmute(self) }
  }
}

impl OptVal for uint {
  fn to_c_repr(self) -> *c_void {
    unsafe { mem::transmute(self) }
  }
}

impl OptVal for bool {
  fn to_c_repr(self) -> *c_void {
    if self {
      1u.to_c_repr()
    } else {
      0u.to_c_repr()
    }
  }
}

impl<'a> OptVal for &'a str {
  fn to_c_repr(self) -> *c_void {
    self.to_c_str().as_bytes().as_ptr() as *c_void
  }
}

macro_rules! DEFOPT(
  ($name:ident, $ty:ident, $num:expr) => (#[allow(dead_code)] pub static $name: Opt = $ty + $num;);
)

macro_rules! ALIAS(
  ($name:ident, $to:ident) => (#[allow(dead_code)] pub static $name: Opt = $to;);
)

DEFOPT!(FILE,                   OBJECTPOINT,     1)
DEFOPT!(URL,                    OBJECTPOINT,     2)
DEFOPT!(PORT,                   LONG,            3)
DEFOPT!(PROXY,                  OBJECTPOINT,     4)
DEFOPT!(USERPWD,                OBJECTPOINT,     5)
DEFOPT!(PROXYUSERPWD,           OBJECTPOINT,     6)
DEFOPT!(RANGE,                  OBJECTPOINT,     7)
/* 8: not used */
DEFOPT!(INFILE,                 OBJECTPOINT,     9)
DEFOPT!(ERRORBUFFER,            OBJECTPOINT,    10)
DEFOPT!(WRITEFUNCTION,          FUNCTIONPOINT,  11)
DEFOPT!(READFUNCTION,           FUNCTIONPOINT,  12)
DEFOPT!(TIMEOUT,                LONG,           13)
DEFOPT!(INFILESIZE,             LONG,           14)
DEFOPT!(POSTFIELDS,             OBJECTPOINT,    15)
DEFOPT!(REFERER,                OBJECTPOINT,    16)
DEFOPT!(FTPPORT,                OBJECTPOINT,    17)
DEFOPT!(USERAGENT,              OBJECTPOINT,    18)
DEFOPT!(LOW_SPEED_LIMIT,        LONG,           19)
DEFOPT!(LOW_SPEED_TIME,         LONG,           20)
DEFOPT!(RESUME_FROM,            LONG,           21)
DEFOPT!(COOKIE,                 OBJECTPOINT,    22)
DEFOPT!(HTTPHEADER,             OBJECTPOINT,    23)
DEFOPT!(HTTPPOST,               OBJECTPOINT,    24)
DEFOPT!(SSLCERT,                OBJECTPOINT,    25)
DEFOPT!(KEYPASSWD,              OBJECTPOINT,    26)
DEFOPT!(CRLF,                   LONG,           27)
DEFOPT!(QUOTE,                  OBJECTPOINT,    28)
DEFOPT!(WRITEHEADER,            OBJECTPOINT,    29)
/* 30: not used */
DEFOPT!(COOKIEFILE,             OBJECTPOINT,    31)
DEFOPT!(SSLVERSION,             LONG,           32)
DEFOPT!(TIMECONDITION,          LONG,           33)
DEFOPT!(TIMEVALUE,              LONG,           34)
/* 35: not used */
DEFOPT!(CUSTOMREQUEST,          OBJECTPOINT,    36)
DEFOPT!(STDERR,                 OBJECTPOINT,    37)
/* 38: not used */
DEFOPT!(POSTQUOTE,              OBJECTPOINT,    39)
DEFOPT!(WRITEINFO,              OBJECTPOINT,    40)
DEFOPT!(VERBOSE,                LONG,           41)
DEFOPT!(HEADER,                 LONG,           42)
DEFOPT!(NOPROGRESS,             LONG,           43)
DEFOPT!(NOBODY,                 LONG,           44)
DEFOPT!(FAILONERROR,            LONG,           45)
DEFOPT!(UPLOAD,                 LONG,           46)
DEFOPT!(POST,                   LONG,           47)
DEFOPT!(DIRLISTONLY,            LONG,           48)
DEFOPT!(APPEND,                 LONG,           50)
DEFOPT!(NETRC,                  LONG,           51)
DEFOPT!(FOLLOWLOCATION,         LONG,           52)
DEFOPT!(TRANSFERTEXT,           LONG,           53)
DEFOPT!(PUT,                    LONG,           54)
/* 55: not used */
DEFOPT!(PROGRESSFUNCTION,       FUNCTIONPOINT,  56)
DEFOPT!(PROGRESSDATA,           OBJECTPOINT,    57)
DEFOPT!(AUTOREFERER,            LONG,           58)
DEFOPT!(PROXYPORT,              LONG,           59)
DEFOPT!(POSTFIELDSIZE,          LONG,           60)
DEFOPT!(HTTPPROXYTUNNEL,        LONG,           61)
DEFOPT!(INTERFACE,              OBJECTPOINT,    62)
DEFOPT!(KRBLEVEL,               OBJECTPOINT,    63)
DEFOPT!(SSL_VERIFYPEER,         LONG,           64)
DEFOPT!(CAINFO,                 OBJECTPOINT,    65)
/* 66: not used */
/* 67: not used */
DEFOPT!(MAXREDIRS,                  LONG,           68)
DEFOPT!(FILETIME,                   LONG,           69)
DEFOPT!(TELNETOPTIONS,              OBJECTPOINT,    70)
DEFOPT!(MAXCONNECTS,                LONG,           71)
DEFOPT!(CLOSEPOLICY,                LONG,           72)
/* 73: not used */
DEFOPT!(FRESH_CONNECT,              LONG,           74)
DEFOPT!(FORBID_REUSE,               LONG,           75)
DEFOPT!(RANDOM_FILE,                OBJECTPOINT,    76)
DEFOPT!(EGDSOCKET,                  OBJECTPOINT,    77)
DEFOPT!(CONNECTTIMEOUT,             LONG,           78)
DEFOPT!(HEADERFUNCTION,             FUNCTIONPOINT,  79)
DEFOPT!(HTTPGET,                    LONG,           80)
DEFOPT!(SSL_VERIFYHOST,             LONG,           81)
DEFOPT!(COOKIEJAR,                  OBJECTPOINT,    82)
DEFOPT!(SSL_CIPHER_LIST,            OBJECTPOINT,    83)
DEFOPT!(HTTP_VERSION,               LONG,           84)
DEFOPT!(FTP_USE_EPSV,               LONG,           85)
DEFOPT!(SSLCERTTYPE,                OBJECTPOINT,    86)
DEFOPT!(SSLKEY,                     OBJECTPOINT,    87)
DEFOPT!(SSLKEYTYPE,                 OBJECTPOINT,    88)
DEFOPT!(SSLENGINE,                  OBJECTPOINT,    89)
DEFOPT!(SSLENGINE_DEFAULT,          LONG,           90)
DEFOPT!(DNS_USE_GLOBAL_CACHE,       LONG,           91)
DEFOPT!(DNS_CACHE_TIMEOUT,          LONG,           92)
DEFOPT!(PREQUOTE,                   OBJECTPOINT,    93)
DEFOPT!(DEBUGFUNCTION,              FUNCTIONPOINT,  94)
DEFOPT!(DEBUGDATA,                  OBJECTPOINT,    95)
DEFOPT!(COOKIESESSION,              LONG,           96)
DEFOPT!(CAPATH,                     OBJECTPOINT,    97)
DEFOPT!(BUFFERSIZE,                 LONG,           98)
DEFOPT!(NOSIGNAL,                   LONG,           99)
DEFOPT!(SHARE,                      OBJECTPOINT,   100)
DEFOPT!(PROXYTYPE,                  LONG,          101)
DEFOPT!(ACCEPT_ENCODING,            OBJECTPOINT,   102)
DEFOPT!(PRIVATE,                    OBJECTPOINT,   103)
DEFOPT!(HTTP200ALIASES,             OBJECTPOINT,   104)
DEFOPT!(UNRESTRICTED_AUTH,          LONG,          105)
DEFOPT!(FTP_USE_EPRT,               LONG,          106)
DEFOPT!(HTTPAUTH,                   LONG,          107)
DEFOPT!(SSL_CTX_FUNCTION,           FUNCTIONPOINT, 108)
DEFOPT!(SSL_CTX_DATA,               OBJECTPOINT,   109)
DEFOPT!(FTP_CREATE_MISSING_DIRS,    LONG,          110)
DEFOPT!(PROXYAUTH,                  LONG,          111)
DEFOPT!(FTP_RESPONSE_TIMEOUT,       LONG,          112)
DEFOPT!(IPRESOLVE,                  LONG,          113)
DEFOPT!(MAXFILESIZE,                LONG,          114)
DEFOPT!(INFILESIZE_LARGE,           OFF_T,         115)
DEFOPT!(RESUME_FROM_LARGE,          OFF_T,         116)
DEFOPT!(MAXFILESIZE_LARGE,          OFF_T,         117)
DEFOPT!(NETRC_FILE,                 OBJECTPOINT,   118)
DEFOPT!(USE_SSL,                    LONG,          119)
DEFOPT!(POSTFIELDSIZE_LARGE,        OFF_T,         120)
DEFOPT!(TCP_NODELAY,                LONG,          121)
/* 122 - 128: not used */
DEFOPT!(FTPSSLAUTH,                 LONG,          129)
DEFOPT!(IOCTLFUNCTION,              FUNCTIONPOINT, 130)
DEFOPT!(IOCTLDATA,                  OBJECTPOINT,   131)
/* 132, 133: not used */
DEFOPT!(FTP_ACCOUNT,                OBJECTPOINT,   134)
DEFOPT!(COOKIELIST,                 OBJECTPOINT,   135)
DEFOPT!(IGNORE_CONTENT_LENGTH,      LONG,          136)
DEFOPT!(FTP_SKIP_PASV_IP,           LONG,          137)
DEFOPT!(FTP_FILEMETHOD,             LONG,          138)
DEFOPT!(LOCALPORT,                  LONG,          139)
DEFOPT!(LOCALPORTRANGE,             LONG,          140)
DEFOPT!(CONNECT_ONLY,               LONG,          141)
DEFOPT!(CONV_FROM_NETWORK_FUNCTION, FUNCTIONPOINT, 142)
DEFOPT!(CONV_TO_NETWORK_FUNCTION,   FUNCTIONPOINT, 143)
DEFOPT!(CONV_FROM_UTF8_FUNCTION,    FUNCTIONPOINT, 144)

// Option aliases
ALIAS!(READDATA,   INFILE)
ALIAS!(WRITEDATA,  FILE)
ALIAS!(HEADERDATA, WRITEHEADER)
