#![allow(non_camel_case_types, raw_pointer_derive)]

extern crate libc;
#[cfg(not(target_env = "msvc"))]
extern crate libz_sys;
#[cfg(all(unix, not(target_os = "macos")))]
extern crate openssl_sys;

use libc::{c_void, c_int, c_char, c_uint, c_long};

pub type CURLINFO = c_int;
pub type CURL = c_void;
pub type curl_slist = c_void;
pub type CURLoption = c_int;

#[repr(C)]
#[derive(Clone, Copy)]
pub enum CURLversion {
    CURL_VERSION_FIRST,
    CURL_VERSION_SECOND,
    CURL_VERSION_THIRD,
    CURL_VERSION_FOURTH,
    CURL_VERSION_LAST /* never actually use this */
}

#[repr(C)]
#[derive(Copy)]
pub struct curl_version_info_data {
    pub age: CURLversion,

    pub version: *const c_char,
    pub version_num: c_uint,
    pub host: *const c_char,
    pub features: c_int,
    pub ssl_version: *const c_char,

    pub ssl_version_num: c_long,

    pub libz_version: *const c_char,

    /* protocols is terminated by an entry with a NULL protoname */
    pub protocols: *const *const c_char,

    /* The fields below this were added in CURL_VERSION_SECOND */
    pub ares: *const c_char,
    pub ares_num: c_int,

    /* This field was added in CURL_VERSION_THIRD */
    pub libidn: *const c_char,

    /* These field were added in CURL_VERSION_FOURTH */
    pub iconv_ver_num: c_int,
    pub libssh_version: *const c_char,
}

impl Clone for curl_version_info_data {
    fn clone(&self) -> Self { *self }
}

pub const CURL_READFUNC_ABORT: c_int = 0x10000000;

pub const CURLINFO_STRING: c_int   = 0x100000;
pub const CURLINFO_LONG: c_int     = 0x200000;
pub const CURLINFO_DOUBLE: c_int   = 0x300000;
pub const CURLINFO_SLIST:  c_int   = 0x400000;
pub const CURLINFO_MASK: c_int     = 0x0fffff;
pub const CURLINFO_TYPEMASK: c_int = 0xf00000;

pub const CURLINFO_EFFECTIVE_URL: CURLINFO = CURLINFO_STRING + 1;
pub const CURLINFO_RESPONSE_CODE: CURLINFO = CURLINFO_LONG + 2;
pub const CURLINFO_TOTAL_TIME: CURLINFO = CURLINFO_DOUBLE + 5;

pub const CURLOPTTYPE_LONG: c_int          = 0;
pub const CURLOPTTYPE_OBJECTPOINT: c_int   = 10_000;
pub const CURLOPTTYPE_FUNCTIONPOINT: c_int = 20_000;
pub const CURLOPTTYPE_OFF_T: c_int         = 30_000;

pub const CURL_VERSION_NOW: CURLversion    = CURLversion::CURL_VERSION_FOURTH;
pub const CURL_VERSION_IPV6:         c_int = (1 << 0);
pub const CURL_VERSION_KERBEROS4:    c_int = (1 << 1);
pub const CURL_VERSION_SSL:          c_int = (1 << 2);
pub const CURL_VERSION_LIBZ:         c_int = (1 << 3);
pub const CURL_VERSION_NTLM:         c_int = (1 << 4);
pub const CURL_VERSION_GSSNEGOTIATE: c_int = (1 << 5);
pub const CURL_VERSION_DEBUG:        c_int = (1 << 6);
pub const CURL_VERSION_ASYNCHDNS:    c_int = (1 << 7);
pub const CURL_VERSION_SPNEGO:       c_int = (1 << 8);
pub const CURL_VERSION_LARGEFILE:    c_int = (1 << 9);
pub const CURL_VERSION_IDN:          c_int = (1 << 10);
pub const CURL_VERSION_SSPI:         c_int = (1 << 11);
pub const CURL_VERSION_CONV:         c_int = (1 << 12);
pub const CURL_VERSION_CURLDEBUG:    c_int = (1 << 13);
pub const CURL_VERSION_TLSAUTH_SRP:  c_int = (1 << 14);
pub const CURL_VERSION_NTLM_WB:      c_int = (1 << 15);
pub const CURL_VERSION_HTTP2:        c_int = (1 << 16);

#[repr(C)]
#[derive(Copy, Clone, Debug)]
pub enum CURLcode {
    CURLE_OK = 0,
    CURLE_UNSUPPORTED_PROTOCOL,    /* 1 */
    CURLE_FAILED_INIT,             /* 2 */
    CURLE_URL_MALFORMAT,           /* 3 */
    CURLE_NOT_BUILT_IN,            /* 4 - [was obsoleted in August 2007 for
                                7.17.0, reused in April 2011 for 7.21.5] */
    CURLE_COULDNT_RESOLVE_PROXY,   /* 5 */
    CURLE_COULDNT_RESOLVE_HOST,    /* 6 */
    CURLE_COULDNT_CONNECT,         /* 7 */
    CURLE_FTP_WEIRD_SERVER_REPLY,  /* 8 */
    CURLE_REMOTE_ACCESS_DENIED,    /* 9 a service was denied by the server
                                due to lack of access - when login fails
                                this is not returned. */
    CURLE_FTP_ACCEPT_FAILED,       /* 10 - [was obsoleted in April 2006 for
                                7.15.4, reused in Dec 2011 for 7.24.0]*/
    CURLE_FTP_WEIRD_PASS_REPLY,    /* 11 */
    CURLE_FTP_ACCEPT_TIMEOUT,      /* 12 - timeout occurred accepting server
                                [was obsoleted in August 2007 for 7.17.0,
                                reused in Dec 2011 for 7.24.0]*/
    CURLE_FTP_WEIRD_PASV_REPLY,    /* 13 */
    CURLE_FTP_WEIRD_227_FORMAT,    /* 14 */
    CURLE_FTP_CANT_GET_HOST,       /* 15 */
    CURLE_OBSOLETE16,              /* 16 - NOT USED */
    CURLE_FTP_COULDNT_SET_TYPE,    /* 17 */
    CURLE_PARTIAL_FILE,            /* 18 */
    CURLE_FTP_COULDNT_RETR_FILE,   /* 19 */
    CURLE_OBSOLETE20,              /* 20 - NOT USED */
    CURLE_QUOTE_ERROR,             /* 21 - quote command failure */
    CURLE_HTTP_RETURNED_ERROR,     /* 22 */
    CURLE_WRITE_ERROR,             /* 23 */
    CURLE_OBSOLETE24,              /* 24 - NOT USED */
    CURLE_UPLOAD_FAILED,           /* 25 - failed upload "command" */
    CURLE_READ_ERROR,              /* 26 - couldn't open/read from file */
    CURLE_OUT_OF_MEMORY,           /* 27 */
    /* Note: CURLE_OUT_OF_MEMORY may sometimes indicate a conversion error
       instead of a memory allocation error if CURL_DOES_CONVERSIONS
       is defined
       */
    CURLE_OPERATION_TIMEDOUT,      /* 28 - the timeout time was reached */
    CURLE_OBSOLETE29,              /* 29 - NOT USED */
    CURLE_FTP_PORT_FAILED,         /* 30 - FTP PORT operation failed */
    CURLE_FTP_COULDNT_USE_REST,    /* 31 - the REST command failed */
    CURLE_OBSOLETE32,              /* 32 - NOT USED */
    CURLE_RANGE_ERROR,             /* 33 - RANGE "command" didn't work */
    CURLE_HTTP_POST_ERROR,         /* 34 */
    CURLE_SSL_CONNECT_ERROR,       /* 35 - wrong when connecting with SSL */
    CURLE_BAD_DOWNLOAD_RESUME,     /* 36 - couldn't resume download */
    CURLE_FILE_COULDNT_READ_FILE,  /* 37 */
    CURLE_LDAP_CANNOT_BIND,        /* 38 */
    CURLE_LDAP_SEARCH_FAILED,      /* 39 */
    CURLE_OBSOLETE40,              /* 40 - NOT USED */
    CURLE_FUNCTION_NOT_FOUND,      /* 41 */
    CURLE_ABORTED_BY_CALLBACK,     /* 42 */
    CURLE_BAD_FUNCTION_ARGUMENT,   /* 43 */
    CURLE_OBSOLETE44,              /* 44 - NOT USED */
    CURLE_INTERFACE_FAILED,        /* 45 - CURLOPT_INTERFACE failed */
    CURLE_OBSOLETE46,              /* 46 - NOT USED */
    CURLE_TOO_MANY_REDIRECTS ,     /* 47 - catch endless re-direct loops */
    CURLE_UNKNOWN_OPTION,          /* 48 - User specified an unknown option */
    CURLE_TELNET_OPTION_SYNTAX ,   /* 49 - Malformed telnet option */
    CURLE_OBSOLETE50,              /* 50 - NOT USED */
    CURLE_PEER_FAILED_VERIFICATION, /* 51 - peer's certificate or fingerprint
                                 wasn't verified fine */
    CURLE_GOT_NOTHING,             /* 52 - when this is a specific error */
    CURLE_SSL_ENGINE_NOTFOUND,     /* 53 - SSL crypto engine not found */
    CURLE_SSL_ENGINE_SETFAILED,    /* 54 - can not set SSL crypto engine as
                                default */
    CURLE_SEND_ERROR,              /* 55 - failed sending network data */
    CURLE_RECV_ERROR,              /* 56 - failure in receiving network data */
    CURLE_OBSOLETE57,              /* 57 - NOT IN USE */
    CURLE_SSL_CERTPROBLEM,         /* 58 - problem with the local certificate */
    CURLE_SSL_CIPHER,              /* 59 - couldn't use specified cipher */
    CURLE_SSL_CACERT,              /* 60 - problem with the CA cert (path?) */
    CURLE_BAD_CONTENT_ENCODING,    /* 61 - Unrecognized/bad encoding */
    CURLE_LDAP_INVALID_URL,        /* 62 - Invalid LDAP URL */
    CURLE_FILESIZE_EXCEEDED,       /* 63 - Maximum file size exceeded */
    CURLE_USE_SSL_FAILED,          /* 64 - Requested FTP SSL level failed */
    CURLE_SEND_FAIL_REWIND,        /* 65 - Sending the data requires a rewind
                                that failed */
    CURLE_SSL_ENGINE_INITFAILED,   /* 66 - failed to initialise ENGINE */
    CURLE_LOGIN_DENIED,            /* 67 - user, password or similar was not
                                accepted and we failed to login */
    CURLE_TFTP_NOTFOUND,           /* 68 - file not found on server */
    CURLE_TFTP_PERM,               /* 69 - permission problem on server */
    CURLE_REMOTE_DISK_FULL,        /* 70 - out of disk space on server */
    CURLE_TFTP_ILLEGAL,            /* 71 - Illegal TFTP operation */
    CURLE_TFTP_UNKNOWNID,          /* 72 - Unknown transfer ID */
    CURLE_REMOTE_FILE_EXISTS,      /* 73 - File already exists */
    CURLE_TFTP_NOSUCHUSER,         /* 74 - No such user */
    CURLE_CONV_FAILED,             /* 75 - conversion failed */
    CURLE_CONV_REQD,               /* 76 - caller must register conversion
                                callbacks using curl_easy_setopt options
                                CURLOPT_CONV_FROM_NETWORK_FUNCTION,
                                CURLOPT_CONV_TO_NETWORK_FUNCTION, and
                                CURLOPT_CONV_FROM_UTF8_FUNCTION */
    CURLE_SSL_CACERT_BADFILE,      /* 77 - could not load CACERT file, missing
                                or wrong format */
    CURLE_REMOTE_FILE_NOT_FOUND,   /* 78 - remote file not found */
    CURLE_SSH,                     /* 79 - error from the SSH layer, somewhat
                                generic so the error message will be of
                                interest when this has happened */

    CURLE_SSL_SHUTDOWN_FAILED,     /* 80 - Failed to shut down the SSL
                                connection */
    CURLE_AGAIN,                   /* 81 - socket is not ready for send/recv,
                                wait till it's ready and try again (Added
                                in 7.18.2) */
    CURLE_SSL_CRL_BADFILE,         /* 82 - could not load CRL file, missing or
                                wrong format (Added in 7.19.0) */
    CURLE_SSL_ISSUER_ERROR,        /* 83 - Issuer check failed.  (Added in
                                7.19.0) */
    CURLE_FTP_PRET_FAILED,         /* 84 - a PRET command failed */
    CURLE_RTSP_CSEQ_ERROR,         /* 85 - mismatch of RTSP CSeq numbers */
    CURLE_RTSP_SESSION_ERROR,      /* 86 - mismatch of RTSP Session Ids */
    CURLE_FTP_BAD_FILE_LIST,       /* 87 - unable to parse FTP file list */
    CURLE_CHUNK_FAILED,            /* 88 - chunk callback reported error */
    CURLE_NO_CONNECTION_AVAILABLE, /* 89 - No connection available, the
                                session will be queued */
    CURLE_LAST /* never use! */
}

macro_rules! DEFOPT {
    ($name:ident, $ty:ident, $num:expr) => (
        #[allow(dead_code)]
        pub const $name: CURLoption = $ty + $num;
    )
}

macro_rules! ALIAS {
    ($name:ident, $to:ident) => (
        #[allow(dead_code)]
        pub const $name: CURLoption = $to;
    )
}

DEFOPT!(CURLOPT_FILE,                   CURLOPTTYPE_OBJECTPOINT,     1);
DEFOPT!(CURLOPT_URL,                    CURLOPTTYPE_OBJECTPOINT,     2);
DEFOPT!(CURLOPT_PORT,                   CURLOPTTYPE_LONG,            3);
DEFOPT!(CURLOPT_PROXY,                  CURLOPTTYPE_OBJECTPOINT,     4);
DEFOPT!(CURLOPT_USERPWD,                CURLOPTTYPE_OBJECTPOINT,     5);
DEFOPT!(CURLOPT_PROXYUSERPWD,           CURLOPTTYPE_OBJECTPOINT,     6);
DEFOPT!(CURLOPT_RANGE,                  CURLOPTTYPE_OBJECTPOINT,     7);
/* 8: not used */
DEFOPT!(CURLOPT_INFILE,                 CURLOPTTYPE_OBJECTPOINT,     9);
DEFOPT!(CURLOPT_ERRORBUFFER,            CURLOPTTYPE_OBJECTPOINT,    10);
DEFOPT!(CURLOPT_WRITEFUNCTION,          CURLOPTTYPE_FUNCTIONPOINT,  11);
DEFOPT!(CURLOPT_READFUNCTION,           CURLOPTTYPE_FUNCTIONPOINT,  12);
DEFOPT!(CURLOPT_TIMEOUT,                CURLOPTTYPE_LONG,           13);
DEFOPT!(CURLOPT_INFILESIZE,             CURLOPTTYPE_LONG,           14);
DEFOPT!(CURLOPT_POSTFIELDS,             CURLOPTTYPE_OBJECTPOINT,    15);
DEFOPT!(CURLOPT_REFERER,                CURLOPTTYPE_OBJECTPOINT,    16);
DEFOPT!(CURLOPT_FTPPORT,                CURLOPTTYPE_OBJECTPOINT,    17);
DEFOPT!(CURLOPT_USERAGENT,              CURLOPTTYPE_OBJECTPOINT,    18);
DEFOPT!(CURLOPT_LOW_SPEED_LIMIT,        CURLOPTTYPE_LONG,           19);
DEFOPT!(CURLOPT_LOW_SPEED_TIME,         CURLOPTTYPE_LONG,           20);
DEFOPT!(CURLOPT_RESUME_FROM,            CURLOPTTYPE_LONG,           21);
DEFOPT!(CURLOPT_COOKIE,                 CURLOPTTYPE_OBJECTPOINT,    22);
DEFOPT!(CURLOPT_HTTPHEADER,             CURLOPTTYPE_OBJECTPOINT,    23);
DEFOPT!(CURLOPT_HTTPPOST,               CURLOPTTYPE_OBJECTPOINT,    24);
DEFOPT!(CURLOPT_SSLCERT,                CURLOPTTYPE_OBJECTPOINT,    25);
DEFOPT!(CURLOPT_KEYPASSWD,              CURLOPTTYPE_OBJECTPOINT,    26);
DEFOPT!(CURLOPT_CRLF,                   CURLOPTTYPE_LONG,           27);
DEFOPT!(CURLOPT_QUOTE,                  CURLOPTTYPE_OBJECTPOINT,    28);
DEFOPT!(CURLOPT_WRITEHEADER,            CURLOPTTYPE_OBJECTPOINT,    29);
/* 30: not used */
DEFOPT!(CURLOPT_COOKIEFILE,             CURLOPTTYPE_OBJECTPOINT,    31);
DEFOPT!(CURLOPT_SSLVERSION,             CURLOPTTYPE_LONG,           32);
DEFOPT!(CURLOPT_TIMECONDITION,          CURLOPTTYPE_LONG,           33);
DEFOPT!(CURLOPT_TIMEVALUE,              CURLOPTTYPE_LONG,           34);
/* 35: not used */
DEFOPT!(CURLOPT_CUSTOMREQUEST,          CURLOPTTYPE_OBJECTPOINT,    36);
DEFOPT!(CURLOPT_STDERR,                 CURLOPTTYPE_OBJECTPOINT,    37);
/* 38: not used */
DEFOPT!(CURLOPT_POSTQUOTE,              CURLOPTTYPE_OBJECTPOINT,    39);
DEFOPT!(CURLOPT_WRITEINFO,              CURLOPTTYPE_OBJECTPOINT,    40);
DEFOPT!(CURLOPT_VERBOSE,                CURLOPTTYPE_LONG,           41);
DEFOPT!(CURLOPT_HEADER,                 CURLOPTTYPE_LONG,           42);
DEFOPT!(CURLOPT_NOPROGRESS,             CURLOPTTYPE_LONG,           43);
DEFOPT!(CURLOPT_NOBODY,                 CURLOPTTYPE_LONG,           44);
DEFOPT!(CURLOPT_FAILONERROR,            CURLOPTTYPE_LONG,           45);
DEFOPT!(CURLOPT_UPLOAD,                 CURLOPTTYPE_LONG,           46);
DEFOPT!(CURLOPT_POST,                   CURLOPTTYPE_LONG,           47);
DEFOPT!(CURLOPT_DIRLISTONLY,            CURLOPTTYPE_LONG,           48);
DEFOPT!(CURLOPT_APPEND,                 CURLOPTTYPE_LONG,           50);
DEFOPT!(CURLOPT_NETRC,                  CURLOPTTYPE_LONG,           51);
DEFOPT!(CURLOPT_FOLLOWLOCATION,         CURLOPTTYPE_LONG,           52);
DEFOPT!(CURLOPT_TRANSFERTEXT,           CURLOPTTYPE_LONG,           53);
DEFOPT!(CURLOPT_PUT,                    CURLOPTTYPE_LONG,           54);
/* 55: not used */
DEFOPT!(CURLOPT_PROGRESSFUNCTION,       CURLOPTTYPE_FUNCTIONPOINT,  56);
DEFOPT!(CURLOPT_PROGRESSDATA,           CURLOPTTYPE_OBJECTPOINT,    57);
DEFOPT!(CURLOPT_AUTOREFERER,            CURLOPTTYPE_LONG,           58);
DEFOPT!(CURLOPT_PROXYPORT,              CURLOPTTYPE_LONG,           59);
DEFOPT!(CURLOPT_POSTFIELDSIZE,          CURLOPTTYPE_LONG,           60);
DEFOPT!(CURLOPT_HTTPPROXYTUNNEL,        CURLOPTTYPE_LONG,           61);
DEFOPT!(CURLOPT_INTERFACE,              CURLOPTTYPE_OBJECTPOINT,    62);
DEFOPT!(CURLOPT_KRBLEVEL,               CURLOPTTYPE_OBJECTPOINT,    63);
DEFOPT!(CURLOPT_SSL_VERIFYPEER,         CURLOPTTYPE_LONG,           64);
DEFOPT!(CURLOPT_CAINFO,                 CURLOPTTYPE_OBJECTPOINT,    65);
/* 66: not used */
/* 67: not used */
DEFOPT!(CURLOPT_MAXREDIRS,                  CURLOPTTYPE_LONG,           68);
DEFOPT!(CURLOPT_FILETIME,                   CURLOPTTYPE_LONG,           69);
DEFOPT!(CURLOPT_TELNETOPTIONS,              CURLOPTTYPE_OBJECTPOINT,    70);
DEFOPT!(CURLOPT_MAXCONNECTS,                CURLOPTTYPE_LONG,           71);
DEFOPT!(CURLOPT_CLOSEPOLICY,                CURLOPTTYPE_LONG,           72);
/* 73: not used */
DEFOPT!(CURLOPT_FRESH_CONNECT,              CURLOPTTYPE_LONG,           74);
DEFOPT!(CURLOPT_FORBID_REUSE,               CURLOPTTYPE_LONG,           75);
DEFOPT!(CURLOPT_RANDOM_FILE,                CURLOPTTYPE_OBJECTPOINT,    76);
DEFOPT!(CURLOPT_EGDSOCKET,                  CURLOPTTYPE_OBJECTPOINT,    77);
DEFOPT!(CURLOPT_CONNECTTIMEOUT,             CURLOPTTYPE_LONG,           78);
DEFOPT!(CURLOPT_HEADERFUNCTION,             CURLOPTTYPE_FUNCTIONPOINT,  79);
DEFOPT!(CURLOPT_HTTPGET,                    CURLOPTTYPE_LONG,           80);
DEFOPT!(CURLOPT_SSL_VERIFYHOST,             CURLOPTTYPE_LONG,           81);
DEFOPT!(CURLOPT_COOKIEJAR,                  CURLOPTTYPE_OBJECTPOINT,    82);
DEFOPT!(CURLOPT_SSL_CIPHER_LIST,            CURLOPTTYPE_OBJECTPOINT,    83);
DEFOPT!(CURLOPT_HTTP_VERSION,               CURLOPTTYPE_LONG,           84);
DEFOPT!(CURLOPT_FTP_USE_EPSV,               CURLOPTTYPE_LONG,           85);
DEFOPT!(CURLOPT_SSLCERTTYPE,                CURLOPTTYPE_OBJECTPOINT,    86);
DEFOPT!(CURLOPT_SSLKEY,                     CURLOPTTYPE_OBJECTPOINT,    87);
DEFOPT!(CURLOPT_SSLKEYTYPE,                 CURLOPTTYPE_OBJECTPOINT,    88);
DEFOPT!(CURLOPT_SSLENGINE,                  CURLOPTTYPE_OBJECTPOINT,    89);
DEFOPT!(CURLOPT_SSLENGINE_DEFAULT,          CURLOPTTYPE_LONG,           90);
DEFOPT!(CURLOPT_DNS_USE_GLOBAL_CACHE,       CURLOPTTYPE_LONG,           91);
DEFOPT!(CURLOPT_DNS_CACHE_TIMEOUT,          CURLOPTTYPE_LONG,           92);
DEFOPT!(CURLOPT_PREQUOTE,                   CURLOPTTYPE_OBJECTPOINT,    93);
DEFOPT!(CURLOPT_DEBUGFUNCTION,              CURLOPTTYPE_FUNCTIONPOINT,  94);
DEFOPT!(CURLOPT_DEBUGDATA,                  CURLOPTTYPE_OBJECTPOINT,    95);
DEFOPT!(CURLOPT_COOKIESESSION,              CURLOPTTYPE_LONG,           96);
DEFOPT!(CURLOPT_CAPATH,                     CURLOPTTYPE_OBJECTPOINT,    97);
DEFOPT!(CURLOPT_BUFFERSIZE,                 CURLOPTTYPE_LONG,           98);
DEFOPT!(CURLOPT_NOSIGNAL,                   CURLOPTTYPE_LONG,           99);
DEFOPT!(CURLOPT_SHARE,                      CURLOPTTYPE_OBJECTPOINT,   100);
DEFOPT!(CURLOPT_PROXYTYPE,                  CURLOPTTYPE_LONG,          101);
DEFOPT!(CURLOPT_ACCEPT_ENCODING,            CURLOPTTYPE_OBJECTPOINT,   102);
DEFOPT!(CURLOPT_PRIVATE,                    CURLOPTTYPE_OBJECTPOINT,   103);
DEFOPT!(CURLOPT_HTTP200ALIASES,             CURLOPTTYPE_OBJECTPOINT,   104);
DEFOPT!(CURLOPT_UNRESTRICTED_AUTH,          CURLOPTTYPE_LONG,          105);
DEFOPT!(CURLOPT_FTP_USE_EPRT,               CURLOPTTYPE_LONG,          106);
DEFOPT!(CURLOPT_HTTPAUTH,                   CURLOPTTYPE_LONG,          107);
DEFOPT!(CURLOPT_SSL_CTX_FUNCTION,           CURLOPTTYPE_FUNCTIONPOINT, 108);
DEFOPT!(CURLOPT_SSL_CTX_DATA,               CURLOPTTYPE_OBJECTPOINT,   109);
DEFOPT!(CURLOPT_FTP_CREATE_MISSING_DIRS,    CURLOPTTYPE_LONG,          110);
DEFOPT!(CURLOPT_PROXYAUTH,                  CURLOPTTYPE_LONG,          111);
DEFOPT!(CURLOPT_FTP_RESPONSE_TIMEOUT,       CURLOPTTYPE_LONG,          112);
DEFOPT!(CURLOPT_IPRESOLVE,                  CURLOPTTYPE_LONG,          113);
DEFOPT!(CURLOPT_MAXFILESIZE,                CURLOPTTYPE_LONG,          114);
DEFOPT!(CURLOPT_INFILESIZE_LARGE,           CURLOPTTYPE_OFF_T,         115);
DEFOPT!(CURLOPT_RESUME_FROM_LARGE,          CURLOPTTYPE_OFF_T,         116);
DEFOPT!(CURLOPT_MAXFILESIZE_LARGE,          CURLOPTTYPE_OFF_T,         117);
DEFOPT!(CURLOPT_NETRC_FILE,                 CURLOPTTYPE_OBJECTPOINT,   118);
DEFOPT!(CURLOPT_USE_SSL,                    CURLOPTTYPE_LONG,          119);
DEFOPT!(CURLOPT_POSTFIELDSIZE_LARGE,        CURLOPTTYPE_OFF_T,         120);
DEFOPT!(CURLOPT_TCP_NODELAY,                CURLOPTTYPE_LONG,          121);
/* 122 - 128: not used */
DEFOPT!(CURLOPT_FTPSSLAUTH,                 CURLOPTTYPE_LONG,          129);
DEFOPT!(CURLOPT_IOCTLFUNCTION,              CURLOPTTYPE_FUNCTIONPOINT, 130);
DEFOPT!(CURLOPT_IOCTLDATA,                  CURLOPTTYPE_OBJECTPOINT,   131);
/* 132, CURLOPTTYPE_133: not used */
DEFOPT!(CURLOPT_FTP_ACCOUNT,                CURLOPTTYPE_OBJECTPOINT,   134);
DEFOPT!(CURLOPT_COOKIELIST,                 CURLOPTTYPE_OBJECTPOINT,   135);
DEFOPT!(CURLOPT_IGNORE_CONTENT_LENGTH,      CURLOPTTYPE_LONG,          136);
DEFOPT!(CURLOPT_FTP_SKIP_PASV_IP,           CURLOPTTYPE_LONG,          137);
DEFOPT!(CURLOPT_FTP_FILEMETHOD,             CURLOPTTYPE_LONG,          138);
DEFOPT!(CURLOPT_LOCALPORT,                  CURLOPTTYPE_LONG,          139);
DEFOPT!(CURLOPT_LOCALPORTRANGE,             CURLOPTTYPE_LONG,          140);
DEFOPT!(CURLOPT_CONNECT_ONLY,               CURLOPTTYPE_LONG,          141);
DEFOPT!(CURLOPT_CONV_FROM_NETWORK_FUNCTION, CURLOPTTYPE_FUNCTIONPOINT, 142);
DEFOPT!(CURLOPT_CONV_TO_NETWORK_FUNCTION,   CURLOPTTYPE_FUNCTIONPOINT, 143);
DEFOPT!(CURLOPT_CONV_FROM_UTF8_FUNCTION,    CURLOPTTYPE_FUNCTIONPOINT, 144);
DEFOPT!(CURLOPT_MAX_SEND_SPEED_LARGE,       CURLOPTTYPE_OFF_T,         145);
DEFOPT!(CURLOPT_MAX_RECV_SPEED_LARGE,       CURLOPTTYPE_OFF_T,         146);
DEFOPT!(CURLOPT_FTP_ALTERNATIVE_TO_USER,    CURLOPTTYPE_OBJECTPOINT,   147);
DEFOPT!(CURLOPT_SOCKOPTFUNCTION,            CURLOPTTYPE_FUNCTIONPOINT, 148);
DEFOPT!(CURLOPT_SOCKOPTDATA,                CURLOPTTYPE_OBJECTPOINT,   149);
DEFOPT!(CURLOPT_SSL_SESSIONID_CACHE,        CURLOPTTYPE_LONG,          150);
DEFOPT!(CURLOPT_SSH_AUTH_TYPES,             CURLOPTTYPE_LONG,          151);
DEFOPT!(CURLOPT_SSH_PUBLIC_KEYFILE,         CURLOPTTYPE_OBJECTPOINT,   152);
DEFOPT!(CURLOPT_SSH_PRIVATE_KEYFILE,        CURLOPTTYPE_OBJECTPOINT,   153);
DEFOPT!(CURLOPT_FTP_SSL_CCC,                CURLOPTTYPE_LONG,          154);
DEFOPT!(CURLOPT_TIMEOUT_MS,                 CURLOPTTYPE_LONG,          155);
DEFOPT!(CURLOPT_CONNECTTIMEOUT_MS,          CURLOPTTYPE_LONG,          156);
DEFOPT!(CURLOPT_HTTP_TRANSFER_DECODING,     CURLOPTTYPE_LONG,          157);
DEFOPT!(CURLOPT_HTTP_CONTENT_DECODING,      CURLOPTTYPE_LONG,          158);
DEFOPT!(CURLOPT_NEW_FILE_PERMS,             CURLOPTTYPE_LONG,          159);
DEFOPT!(CURLOPT_NEW_DIRECTORY_PERMS,        CURLOPTTYPE_LONG,          160);
DEFOPT!(CURLOPT_POSTREDIR,                  CURLOPTTYPE_LONG,          161);
DEFOPT!(CURLOPT_SSH_HOST_PUBLIC_KEY_MD5,    CURLOPTTYPE_OBJECTPOINT,   162);
DEFOPT!(CURLOPT_OPENSOCKETFUNCTION,         CURLOPTTYPE_FUNCTIONPOINT, 163);
DEFOPT!(CURLOPT_OPENSOCKETDATA,             CURLOPTTYPE_OBJECTPOINT,   164);
DEFOPT!(CURLOPT_COPYPOSTFIELDS,             CURLOPTTYPE_OBJECTPOINT,   165);
DEFOPT!(CURLOPT_PROXY_TRANSFER_MODE,        CURLOPTTYPE_LONG,          166);
DEFOPT!(CURLOPT_SEEKFUNCTION,               CURLOPTTYPE_FUNCTIONPOINT, 167);
DEFOPT!(CURLOPT_SEEKDATA,                   CURLOPTTYPE_OBJECTPOINT,   168);
DEFOPT!(CURLOPT_CRLFILE,                    CURLOPTTYPE_OBJECTPOINT,   169);
DEFOPT!(CURLOPT_ISSUERCERT,                 CURLOPTTYPE_OBJECTPOINT,   170);
DEFOPT!(CURLOPT_ADDRESS_SCOPE,              CURLOPTTYPE_LONG,          171);
DEFOPT!(CURLOPT_CERTINFO,                   CURLOPTTYPE_LONG,          172);
DEFOPT!(CURLOPT_USERNAME,                   CURLOPTTYPE_OBJECTPOINT,   173);
DEFOPT!(CURLOPT_PASSWORD,                   CURLOPTTYPE_OBJECTPOINT,   174);
DEFOPT!(CURLOPT_PROXYUSERNAME,              CURLOPTTYPE_OBJECTPOINT,   175);
DEFOPT!(CURLOPT_PROXYPASSWORD,              CURLOPTTYPE_OBJECTPOINT,   176);
DEFOPT!(CURLOPT_NOPROXY,                    CURLOPTTYPE_OBJECTPOINT,   177);
DEFOPT!(CURLOPT_TFTP_BLKSIZE,               CURLOPTTYPE_LONG,          178);
DEFOPT!(CURLOPT_SOCKS5_GSSAPI_SERVICE,      CURLOPTTYPE_OBJECTPOINT,   179);
DEFOPT!(CURLOPT_SOCKS5_GSSAPI_NEC,          CURLOPTTYPE_LONG,          180);
DEFOPT!(CURLOPT_PROTOCOLS,                  CURLOPTTYPE_LONG,          181);
DEFOPT!(CURLOPT_REDIR_PROTOCOLS,            CURLOPTTYPE_LONG,          182);
DEFOPT!(CURLOPT_SSH_KNOWNHOSTS,             CURLOPTTYPE_OBJECTPOINT,   183);
DEFOPT!(CURLOPT_SSH_KEYFUNCTION,            CURLOPTTYPE_FUNCTIONPOINT, 184);
DEFOPT!(CURLOPT_SSH_KEYDATA,                CURLOPTTYPE_OBJECTPOINT,   185);
DEFOPT!(CURLOPT_MAIL_FROM,                  CURLOPTTYPE_OBJECTPOINT,   186);
DEFOPT!(CURLOPT_MAIL_RCPT,                  CURLOPTTYPE_OBJECTPOINT,   187);
DEFOPT!(CURLOPT_FTP_USE_PRET,               CURLOPTTYPE_LONG,          188);
DEFOPT!(CURLOPT_RTSP_REQUEST,               CURLOPTTYPE_LONG,          189);
DEFOPT!(CURLOPT_RTSP_SESSION_ID,            CURLOPTTYPE_OBJECTPOINT,   190);
DEFOPT!(CURLOPT_RTSP_STREAM_URI,            CURLOPTTYPE_OBJECTPOINT,   191);
DEFOPT!(CURLOPT_RTSP_TRANSPORT,             CURLOPTTYPE_OBJECTPOINT,   192);
DEFOPT!(CURLOPT_RTSP_CLIENT_CSEQ,           CURLOPTTYPE_LONG,          193);
DEFOPT!(CURLOPT_RTSP_SERVER_CSEQ,           CURLOPTTYPE_LONG,          194);
DEFOPT!(CURLOPT_INTERLEAVEDATA,             CURLOPTTYPE_OBJECTPOINT,   195);
DEFOPT!(CURLOPT_INTERLEAVEFUNCTION,         CURLOPTTYPE_FUNCTIONPOINT, 196);
DEFOPT!(CURLOPT_WILDCARDMATCH,              CURLOPTTYPE_LONG,          197);
DEFOPT!(CURLOPT_CHUNK_BGN_FUNCTION,         CURLOPTTYPE_FUNCTIONPOINT, 198);
DEFOPT!(CURLOPT_CHUNK_END_FUNCTION,         CURLOPTTYPE_FUNCTIONPOINT, 199);
DEFOPT!(CURLOPT_FNMATCH_FUNCTION,           CURLOPTTYPE_FUNCTIONPOINT, 200);
DEFOPT!(CURLOPT_CHUNK_DATA,                 CURLOPTTYPE_OBJECTPOINT,   201);
DEFOPT!(CURLOPT_FNMATCH_DATA,               CURLOPTTYPE_OBJECTPOINT,   202);
DEFOPT!(CURLOPT_RESOLVE,                    CURLOPTTYPE_OBJECTPOINT,   203);
DEFOPT!(CURLOPT_TLSAUTH_USERNAME,           CURLOPTTYPE_OBJECTPOINT,   204);
DEFOPT!(CURLOPT_TLSAUTH_PASSWORD,           CURLOPTTYPE_OBJECTPOINT,   205);
DEFOPT!(CURLOPT_TLSAUTH_TYPE,               CURLOPTTYPE_OBJECTPOINT,   206);
DEFOPT!(CURLOPT_TRANSFER_ENCODING,          CURLOPTTYPE_LONG,          207);
DEFOPT!(CURLOPT_CLOSESOCKETFUNCTION,        CURLOPTTYPE_FUNCTIONPOINT, 208);
DEFOPT!(CURLOPT_CLOSESOCKETDATA,            CURLOPTTYPE_OBJECTPOINT,   209);
DEFOPT!(CURLOPT_GSSAPI_DELEGATION,          CURLOPTTYPE_LONG,          210);
DEFOPT!(CURLOPT_DNS_SERVERS,                CURLOPTTYPE_OBJECTPOINT,   211);
DEFOPT!(CURLOPT_ACCEPTTIMEOUT_MS,           CURLOPTTYPE_LONG,          212);
DEFOPT!(CURLOPT_TCP_KEEPALIVE,              CURLOPTTYPE_LONG,          213);
DEFOPT!(CURLOPT_TCP_KEEPIDLE,               CURLOPTTYPE_LONG,          214);
DEFOPT!(CURLOPT_TCP_KEEPINTVL,              CURLOPTTYPE_LONG,          215);
DEFOPT!(CURLOPT_SSL_OPTIONS,                CURLOPTTYPE_LONG,          216);
DEFOPT!(CURLOPT_MAIL_AUTH,                  CURLOPTTYPE_OBJECTPOINT,   217);
DEFOPT!(CURLOPT_SASL_IR,                    CURLOPTTYPE_LONG,          218);
DEFOPT!(CURLOPT_XFERINFOFUNCTION,           CURLOPTTYPE_FUNCTIONPOINT, 219);
DEFOPT!(CURLOPT_XOAUTH2_BEARER,             CURLOPTTYPE_OBJECTPOINT,   220);
DEFOPT!(CURLOPT_DNS_INTERFACE,              CURLOPTTYPE_OBJECTPOINT,   221);
DEFOPT!(CURLOPT_DNS_LOCAL_IP4,              CURLOPTTYPE_OBJECTPOINT,   222);
DEFOPT!(CURLOPT_DNS_LOCAL_IP6,              CURLOPTTYPE_OBJECTPOINT,   223);
DEFOPT!(CURLOPT_LOGIN_OPTIONS,              CURLOPTTYPE_OBJECTPOINT,   224);
DEFOPT!(CURLOPT_SSL_ENABLE_NPN,             CURLOPTTYPE_LONG,          225);
DEFOPT!(CURLOPT_SSL_ENABLE_ALPN,            CURLOPTTYPE_LONG,          226);
DEFOPT!(CURLOPT_EXPECT_100_TIMEOUT_MS,      CURLOPTTYPE_LONG,          227);
DEFOPT!(CURLOPT_PROXYHEADER,                CURLOPTTYPE_OBJECTPOINT,   228);
DEFOPT!(CURLOPT_HEADEROPT,                  CURLOPTTYPE_LONG,          229);

// Option aliases
ALIAS!(CURLOPT_POST301, CURLOPT_POSTREDIR);
ALIAS!(CURLOPT_SSLKEYPASSWD, CURLOPT_KEYPASSWD);
ALIAS!(CURLOPT_FTPAPPEND, CURLOPT_APPEND);
ALIAS!(CURLOPT_FTPLISTONLY, CURLOPT_DIRLISTONLY);
ALIAS!(CURLOPT_FTP_SSL, CURLOPT_USE_SSL);
ALIAS!(CURLOPT_SSLCERTPASSWD, CURLOPT_KEYPASSWD);
ALIAS!(CURLOPT_KRB4LEVEL, CURLOPT_KRBLEVEL);
ALIAS!(CURLOPT_READDATA, CURLOPT_INFILE);
ALIAS!(CURLOPT_WRITEDATA, CURLOPT_FILE);
ALIAS!(CURLOPT_HEADERDATA, CURLOPT_WRITEHEADER);
ALIAS!(CURLOPT_XFERINFODATA, CURLOPT_PROGRESSDATA);

extern {
    pub fn curl_easy_strerror(code: CURLcode) -> *const c_char;
    pub fn curl_easy_init() -> *mut CURL;
    pub fn curl_easy_setopt(curl: *mut CURL, option: CURLoption, ...) -> CURLcode;
    pub fn curl_easy_perform(curl: *mut CURL) -> CURLcode;
    pub fn curl_easy_cleanup(curl: *mut CURL);
    pub fn curl_easy_getinfo(curl: *const CURL, info: CURLINFO, ...) -> CURLcode;
    pub fn curl_global_cleanup();

    pub fn curl_slist_append(list: *mut curl_slist,
                             val: *const u8) -> *mut curl_slist;
    pub fn curl_slist_free_all(list: *mut curl_slist);

    pub fn curl_version() -> *const c_char;
    pub fn curl_version_info(t: CURLversion) -> *mut curl_version_info_data;
}
