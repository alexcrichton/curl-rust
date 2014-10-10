use std::path::Path;
use libc::{c_void,c_int};

const LONG: c_int          = 0;
const OBJECTPOINT: c_int   = 10_000;
const FUNCTIONPOINT: c_int = 20_000;
const OFF_T: c_int         = 30_000;

pub type Opt = c_int;

pub trait OptVal {
    fn with_c_repr(self, f: |*const c_void|);
}

impl OptVal for int {
    fn with_c_repr(self, f: |*const c_void|) {
        f(self as *const c_void)
    }
}

impl OptVal for uint {
    fn with_c_repr(self, f: |*const c_void|) {
        f(self as *const c_void)
    }
}

impl OptVal for bool {
    fn with_c_repr(self, f: |*const c_void|) {
        f(self as uint as *const c_void)
    }
}

impl<'a> OptVal for &'a str {
    fn with_c_repr(self, f: |*const c_void|) {
        self.with_c_str(|arg| f(arg as *const c_void))
    }
}

impl<'a> OptVal for &'a Path {
    fn with_c_repr(self, f: |*const c_void|) {
        self.with_c_str(|arg| f(arg as *const c_void))
    }
}

macro_rules! DEFOPT(
    ($name:ident, $ty:ident, $num:expr) => (
        #[allow(dead_code)]
        pub const $name: Opt = $ty + $num;
    )
)

macro_rules! ALIAS(
    ($name:ident, $to:ident) => (
        #[allow(dead_code)]
        pub static $name: Opt = $to;
    )
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
DEFOPT!(MAX_SEND_SPEED_LARGE,       OFF_T,         145)
DEFOPT!(MAX_RECV_SPEED_LARGE,       OFF_T,         146)
DEFOPT!(FTP_ALTERNATIVE_TO_USER,    OBJECTPOINT,   147)
DEFOPT!(SOCKOPTFUNCTION,            FUNCTIONPOINT, 148)
DEFOPT!(SOCKOPTDATA,                OBJECTPOINT,   149)
DEFOPT!(SSL_SESSIONID_CACHE,        LONG,          150)
DEFOPT!(SSH_AUTH_TYPES,             LONG,          151)
DEFOPT!(SSH_PUBLIC_KEYFILE,         OBJECTPOINT,   152)
DEFOPT!(SSH_PRIVATE_KEYFILE,        OBJECTPOINT,   153)
DEFOPT!(FTP_SSL_CCC,                LONG,          154)
DEFOPT!(TIMEOUT_MS,                 LONG,          155)
DEFOPT!(CONNECTTIMEOUT_MS,          LONG,          156)
DEFOPT!(HTTP_TRANSFER_DECODING,     LONG,          157)
DEFOPT!(HTTP_CONTENT_DECODING,      LONG,          158)
DEFOPT!(NEW_FILE_PERMS,             LONG,          159)
DEFOPT!(NEW_DIRECTORY_PERMS,        LONG,          160)
DEFOPT!(POSTREDIR,                  LONG,          161)
DEFOPT!(SSH_HOST_PUBLIC_KEY_MD5,    OBJECTPOINT,   162)
DEFOPT!(OPENSOCKETFUNCTION,         FUNCTIONPOINT, 163)
DEFOPT!(OPENSOCKETDATA,             OBJECTPOINT,   164)
DEFOPT!(COPYPOSTFIELDS,             OBJECTPOINT,   165)
DEFOPT!(PROXY_TRANSFER_MODE,        LONG,          166)
DEFOPT!(SEEKFUNCTION,               FUNCTIONPOINT, 167)
DEFOPT!(SEEKDATA,                   OBJECTPOINT,   168)
DEFOPT!(CRLFILE,                    OBJECTPOINT,   169)
DEFOPT!(ISSUERCERT,                 OBJECTPOINT,   170)
DEFOPT!(ADDRESS_SCOPE,              LONG,          171)
DEFOPT!(CERTINFO,                   LONG,          172)
DEFOPT!(USERNAME,                   OBJECTPOINT,   173)
DEFOPT!(PASSWORD,                   OBJECTPOINT,   174)
DEFOPT!(PROXYUSERNAME,              OBJECTPOINT,   175)
DEFOPT!(PROXYPASSWORD,              OBJECTPOINT,   176)
DEFOPT!(NOPROXY,                    OBJECTPOINT,   177)
DEFOPT!(TFTP_BLKSIZE,               LONG,          178)
DEFOPT!(SOCKS5_GSSAPI_SERVICE,      OBJECTPOINT,   179)
DEFOPT!(SOCKS5_GSSAPI_NEC,          LONG,          180)
DEFOPT!(PROTOCOLS,                  LONG,          181)
DEFOPT!(REDIR_PROTOCOLS,            LONG,          182)
DEFOPT!(SSH_KNOWNHOSTS,             OBJECTPOINT,   183)
DEFOPT!(SSH_KEYFUNCTION,            FUNCTIONPOINT, 184)
DEFOPT!(SSH_KEYDATA,                OBJECTPOINT,   185)
DEFOPT!(MAIL_FROM,                  OBJECTPOINT,   186)
DEFOPT!(MAIL_RCPT,                  OBJECTPOINT,   187)
DEFOPT!(FTP_USE_PRET,               LONG,          188)
DEFOPT!(RTSP_REQUEST,               LONG,          189)
DEFOPT!(RTSP_SESSION_ID,            OBJECTPOINT,   190)
DEFOPT!(RTSP_STREAM_URI,            OBJECTPOINT,   191)
DEFOPT!(RTSP_TRANSPORT,             OBJECTPOINT,   192)
DEFOPT!(RTSP_CLIENT_CSEQ,           LONG,          193)
DEFOPT!(RTSP_SERVER_CSEQ,           LONG,          194)
DEFOPT!(INTERLEAVEDATA,             OBJECTPOINT,   195)
DEFOPT!(INTERLEAVEFUNCTION,         FUNCTIONPOINT, 196)
DEFOPT!(WILDCARDMATCH,              LONG,          197)
DEFOPT!(CHUNK_BGN_FUNCTION,         FUNCTIONPOINT, 198)
DEFOPT!(CHUNK_END_FUNCTION,         FUNCTIONPOINT, 199)
DEFOPT!(FNMATCH_FUNCTION,           FUNCTIONPOINT, 200)
DEFOPT!(CHUNK_DATA,                 OBJECTPOINT,   201)
DEFOPT!(FNMATCH_DATA,               OBJECTPOINT,   202)
DEFOPT!(RESOLVE,                    OBJECTPOINT,   203)
DEFOPT!(TLSAUTH_USERNAME,           OBJECTPOINT,   204)
DEFOPT!(TLSAUTH_PASSWORD,           OBJECTPOINT,   205)
DEFOPT!(TLSAUTH_TYPE,               OBJECTPOINT,   206)
DEFOPT!(TRANSFER_ENCODING,          LONG,          207)
DEFOPT!(CLOSESOCKETFUNCTION,        FUNCTIONPOINT, 208)
DEFOPT!(CLOSESOCKETDATA,            OBJECTPOINT,   209)
DEFOPT!(GSSAPI_DELEGATION,          LONG,          210)
DEFOPT!(DNS_SERVERS,                OBJECTPOINT,   211)
DEFOPT!(ACCEPTTIMEOUT_MS,           LONG,          212)
DEFOPT!(TCP_KEEPALIVE,              LONG,          213)
DEFOPT!(TCP_KEEPIDLE,               LONG,          214)
DEFOPT!(TCP_KEEPINTVL,              LONG,          215)
DEFOPT!(SSL_OPTIONS,                LONG,          216)
DEFOPT!(MAIL_AUTH,                  OBJECTPOINT,   217)
DEFOPT!(SASL_IR,                    LONG,          218)
DEFOPT!(XFERINFOFUNCTION,           FUNCTIONPOINT, 219)
DEFOPT!(XOAUTH2_BEARER,             OBJECTPOINT,   220)
DEFOPT!(DNS_INTERFACE,              OBJECTPOINT,   221)
DEFOPT!(DNS_LOCAL_IP4,              OBJECTPOINT,   222)
DEFOPT!(DNS_LOCAL_IP6,              OBJECTPOINT,   223)
DEFOPT!(LOGIN_OPTIONS,              OBJECTPOINT,   224)
DEFOPT!(SSL_ENABLE_NPN,             LONG,          225)
DEFOPT!(SSL_ENABLE_ALPN,            LONG,          226)
DEFOPT!(EXPECT_100_TIMEOUT_MS,      LONG,          227)
DEFOPT!(PROXYHEADER,                OBJECTPOINT,   228)
DEFOPT!(HEADEROPT,                  LONG,          229)

    // Option aliases
ALIAS!(POST301, POSTREDIR)
ALIAS!(SSLKEYPASSWD, KEYPASSWD)
ALIAS!(FTPAPPEND, APPEND)
ALIAS!(FTPLISTONLY, DIRLISTONLY)
ALIAS!(FTP_SSL, USE_SSL)
ALIAS!(SSLCERTPASSWD, KEYPASSWD)
ALIAS!(KRB4LEVEL, KRBLEVEL)
ALIAS!(READDATA,   INFILE)
ALIAS!(WRITEDATA,  FILE)
ALIAS!(HEADERDATA, WRITEHEADER)
ALIAS!(XFERINFODATA, PROGRESSDATA)
