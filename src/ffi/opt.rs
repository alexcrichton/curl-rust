#![allow(dead_code)]

use std::ffi::CString;
use std::path::Path;
use libc::{c_void};

use curl_ffi as ffi;

pub use curl_ffi::CURLOPT_FILE as FILE;
pub use curl_ffi::CURLOPT_URL as URL;
pub use curl_ffi::CURLOPT_PORT as PORT;
pub use curl_ffi::CURLOPT_PROXY as PROXY;
pub use curl_ffi::CURLOPT_USERPWD as USERPWD;
pub use curl_ffi::CURLOPT_PROXYUSERPWD as PROXYUSERPWD;
pub use curl_ffi::CURLOPT_RANGE as RANGE;
pub use curl_ffi::CURLOPT_INFILE as INFILE;
pub use curl_ffi::CURLOPT_ERRORBUFFER as ERRORBUFFER;
pub use curl_ffi::CURLOPT_WRITEFUNCTION as WRITEFUNCTION;
pub use curl_ffi::CURLOPT_READFUNCTION as READFUNCTION;
pub use curl_ffi::CURLOPT_TIMEOUT as TIMEOUT;
pub use curl_ffi::CURLOPT_INFILESIZE as INFILESIZE;
pub use curl_ffi::CURLOPT_POSTFIELDS as POSTFIELDS;
pub use curl_ffi::CURLOPT_REFERER as REFERER;
pub use curl_ffi::CURLOPT_FTPPORT as FTPPORT;
pub use curl_ffi::CURLOPT_USERAGENT as USERAGENT;
pub use curl_ffi::CURLOPT_LOW_SPEED_LIMIT as LOW_SPEED_LIMIT;
pub use curl_ffi::CURLOPT_LOW_SPEED_TIME as LOW_SPEED_TIME;
pub use curl_ffi::CURLOPT_RESUME_FROM as RESUME_FROM;
pub use curl_ffi::CURLOPT_COOKIE as COOKIE;
pub use curl_ffi::CURLOPT_HTTPHEADER as HTTPHEADER;
pub use curl_ffi::CURLOPT_HTTPPOST as HTTPPOST;
pub use curl_ffi::CURLOPT_SSLCERT as SSLCERT;
pub use curl_ffi::CURLOPT_KEYPASSWD as KEYPASSWD;
pub use curl_ffi::CURLOPT_CRLF as CRLF;
pub use curl_ffi::CURLOPT_QUOTE as QUOTE;
pub use curl_ffi::CURLOPT_WRITEHEADER as WRITEHEADER;
pub use curl_ffi::CURLOPT_COOKIEFILE as COOKIEFILE;
pub use curl_ffi::CURLOPT_SSLVERSION as SSLVERSION;
pub use curl_ffi::CURLOPT_TIMECONDITION as TIMECONDITION;
pub use curl_ffi::CURLOPT_TIMEVALUE as TIMEVALUE;
pub use curl_ffi::CURLOPT_CUSTOMREQUEST as CUSTOMREQUEST;
pub use curl_ffi::CURLOPT_STDERR as STDERR;
pub use curl_ffi::CURLOPT_POSTQUOTE as POSTQUOTE;
pub use curl_ffi::CURLOPT_WRITEINFO as WRITEINFO;
pub use curl_ffi::CURLOPT_VERBOSE as VERBOSE;
pub use curl_ffi::CURLOPT_HEADER as HEADER;
pub use curl_ffi::CURLOPT_NOPROGRESS as NOPROGRESS;
pub use curl_ffi::CURLOPT_NOBODY as NOBODY;
pub use curl_ffi::CURLOPT_FAILONERROR as FAILONERROR;
pub use curl_ffi::CURLOPT_UPLOAD as UPLOAD;
pub use curl_ffi::CURLOPT_POST as POST;
pub use curl_ffi::CURLOPT_DIRLISTONLY as DIRLISTONLY;
pub use curl_ffi::CURLOPT_APPEND as APPEND;
pub use curl_ffi::CURLOPT_NETRC as NETRC;
pub use curl_ffi::CURLOPT_FOLLOWLOCATION as FOLLOWLOCATION;
pub use curl_ffi::CURLOPT_TRANSFERTEXT as TRANSFERTEXT;
pub use curl_ffi::CURLOPT_PUT as PUT;
pub use curl_ffi::CURLOPT_PROGRESSFUNCTION as PROGRESSFUNCTION;
pub use curl_ffi::CURLOPT_PROGRESSDATA as PROGRESSDATA;
pub use curl_ffi::CURLOPT_AUTOREFERER as AUTOREFERER;
pub use curl_ffi::CURLOPT_PROXYPORT as PROXYPORT;
pub use curl_ffi::CURLOPT_POSTFIELDSIZE as POSTFIELDSIZE;
pub use curl_ffi::CURLOPT_HTTPPROXYTUNNEL as HTTPPROXYTUNNEL;
pub use curl_ffi::CURLOPT_INTERFACE as INTERFACE;
pub use curl_ffi::CURLOPT_KRBLEVEL as KRBLEVEL;
pub use curl_ffi::CURLOPT_SSL_VERIFYPEER as SSL_VERIFYPEER;
pub use curl_ffi::CURLOPT_CAINFO as CAINFO;
pub use curl_ffi::CURLOPT_MAXREDIRS as MAXREDIRS;
pub use curl_ffi::CURLOPT_FILETIME as FILETIME;
pub use curl_ffi::CURLOPT_TELNETOPTIONS as TELNETOPTIONS;
pub use curl_ffi::CURLOPT_MAXCONNECTS as MAXCONNECTS;
pub use curl_ffi::CURLOPT_CLOSEPOLICY as CLOSEPOLICY;
pub use curl_ffi::CURLOPT_FRESH_CONNECT as FRESH_CONNECT;
pub use curl_ffi::CURLOPT_FORBID_REUSE as FORBID_REUSE;
pub use curl_ffi::CURLOPT_RANDOM_FILE as RANDOM_FILE;
pub use curl_ffi::CURLOPT_EGDSOCKET as EGDSOCKET;
pub use curl_ffi::CURLOPT_CONNECTTIMEOUT as CONNECTTIMEOUT;
pub use curl_ffi::CURLOPT_HEADERFUNCTION as HEADERFUNCTION;
pub use curl_ffi::CURLOPT_HTTPGET as HTTPGET;
pub use curl_ffi::CURLOPT_SSL_VERIFYHOST as SSL_VERIFYHOST;
pub use curl_ffi::CURLOPT_COOKIEJAR as COOKIEJAR;
pub use curl_ffi::CURLOPT_SSL_CIPHER_LIST as SSL_CIPHER_LIST;
pub use curl_ffi::CURLOPT_HTTP_VERSION as HTTP_VERSION;
pub use curl_ffi::CURLOPT_FTP_USE_EPSV as FTP_USE_EPSV;
pub use curl_ffi::CURLOPT_SSLCERTTYPE as SSLCERTTYPE;
pub use curl_ffi::CURLOPT_SSLKEY as SSLKEY;
pub use curl_ffi::CURLOPT_SSLKEYTYPE as SSLKEYTYPE;
pub use curl_ffi::CURLOPT_SSLENGINE as SSLENGINE;
pub use curl_ffi::CURLOPT_SSLENGINE_DEFAULT as SSLENGINE_DEFAULT;
pub use curl_ffi::CURLOPT_DNS_USE_GLOBAL_CACHE as DNS_USE_GLOBAL_CACHE;
pub use curl_ffi::CURLOPT_DNS_CACHE_TIMEOUT as DNS_CACHE_TIMEOUT;
pub use curl_ffi::CURLOPT_PREQUOTE as PREQUOTE;
pub use curl_ffi::CURLOPT_DEBUGFUNCTION as DEBUGFUNCTION;
pub use curl_ffi::CURLOPT_DEBUGDATA as DEBUGDATA;
pub use curl_ffi::CURLOPT_COOKIESESSION as COOKIESESSION;
pub use curl_ffi::CURLOPT_CAPATH as CAPATH;
pub use curl_ffi::CURLOPT_BUFFERSIZE as BUFFERSIZE;
pub use curl_ffi::CURLOPT_NOSIGNAL as NOSIGNAL;
pub use curl_ffi::CURLOPT_SHARE as SHARE;
pub use curl_ffi::CURLOPT_PROXYTYPE as PROXYTYPE;
pub use curl_ffi::CURLOPT_ACCEPT_ENCODING as ACCEPT_ENCODING;
pub use curl_ffi::CURLOPT_PRIVATE as PRIVATE;
pub use curl_ffi::CURLOPT_HTTP200ALIASES as HTTP200ALIASES;
pub use curl_ffi::CURLOPT_UNRESTRICTED_AUTH as UNRESTRICTED_AUTH;
pub use curl_ffi::CURLOPT_FTP_USE_EPRT as FTP_USE_EPRT;
pub use curl_ffi::CURLOPT_HTTPAUTH as HTTPAUTH;
pub use curl_ffi::CURLOPT_SSL_CTX_FUNCTION as SSL_CTX_FUNCTION;
pub use curl_ffi::CURLOPT_SSL_CTX_DATA as SSL_CTX_DATA;
pub use curl_ffi::CURLOPT_FTP_CREATE_MISSING_DIRS as FTP_CREATE_MISSING_DIRS;
pub use curl_ffi::CURLOPT_PROXYAUTH as PROXYAUTH;
pub use curl_ffi::CURLOPT_FTP_RESPONSE_TIMEOUT as FTP_RESPONSE_TIMEOUT;
pub use curl_ffi::CURLOPT_IPRESOLVE as IPRESOLVE;
pub use curl_ffi::CURLOPT_MAXFILESIZE as MAXFILESIZE;
pub use curl_ffi::CURLOPT_INFILESIZE_LARGE as INFILESIZE_LARGE;
pub use curl_ffi::CURLOPT_RESUME_FROM_LARGE as RESUME_FROM_LARGE;
pub use curl_ffi::CURLOPT_MAXFILESIZE_LARGE as MAXFILESIZE_LARGE;
pub use curl_ffi::CURLOPT_NETRC_FILE as NETRC_FILE;
pub use curl_ffi::CURLOPT_USE_SSL as USE_SSL;
pub use curl_ffi::CURLOPT_POSTFIELDSIZE_LARGE as POSTFIELDSIZE_LARGE;
pub use curl_ffi::CURLOPT_TCP_NODELAY as TCP_NODELAY;
pub use curl_ffi::CURLOPT_FTPSSLAUTH as FTPSSLAUTH;
pub use curl_ffi::CURLOPT_IOCTLFUNCTION as IOCTLFUNCTION;
pub use curl_ffi::CURLOPT_IOCTLDATA as IOCTLDATA;
pub use curl_ffi::CURLOPT_FTP_ACCOUNT as FTP_ACCOUNT;
pub use curl_ffi::CURLOPT_COOKIELIST as COOKIELIST;
pub use curl_ffi::CURLOPT_IGNORE_CONTENT_LENGTH as IGNORE_CONTENT_LENGTH;
pub use curl_ffi::CURLOPT_FTP_SKIP_PASV_IP as FTP_SKIP_PASV_IP;
pub use curl_ffi::CURLOPT_FTP_FILEMETHOD as FTP_FILEMETHOD;
pub use curl_ffi::CURLOPT_LOCALPORT as LOCALPORT;
pub use curl_ffi::CURLOPT_LOCALPORTRANGE as LOCALPORTRANGE;
pub use curl_ffi::CURLOPT_CONNECT_ONLY as CONNECT_ONLY;
pub use curl_ffi::CURLOPT_CONV_FROM_NETWORK_FUNCTION as CONV_FROM_NETWORK_FUNCTION;
pub use curl_ffi::CURLOPT_CONV_TO_NETWORK_FUNCTION as CONV_TO_NETWORK_FUNCTION;
pub use curl_ffi::CURLOPT_CONV_FROM_UTF8_FUNCTION as CONV_FROM_UTF8_FUNCTION;
pub use curl_ffi::CURLOPT_MAX_SEND_SPEED_LARGE as MAX_SEND_SPEED_LARGE;
pub use curl_ffi::CURLOPT_MAX_RECV_SPEED_LARGE as MAX_RECV_SPEED_LARGE;
pub use curl_ffi::CURLOPT_FTP_ALTERNATIVE_TO_USER as FTP_ALTERNATIVE_TO_USER;
pub use curl_ffi::CURLOPT_SOCKOPTFUNCTION as SOCKOPTFUNCTION;
pub use curl_ffi::CURLOPT_SOCKOPTDATA as SOCKOPTDATA;
pub use curl_ffi::CURLOPT_SSL_SESSIONID_CACHE as SSL_SESSIONID_CACHE;
pub use curl_ffi::CURLOPT_SSH_AUTH_TYPES as SSH_AUTH_TYPES;
pub use curl_ffi::CURLOPT_SSH_PUBLIC_KEYFILE as SSH_PUBLIC_KEYFILE;
pub use curl_ffi::CURLOPT_SSH_PRIVATE_KEYFILE as SSH_PRIVATE_KEYFILE;
pub use curl_ffi::CURLOPT_FTP_SSL_CCC as FTP_SSL_CCC;
pub use curl_ffi::CURLOPT_TIMEOUT_MS as TIMEOUT_MS;
pub use curl_ffi::CURLOPT_CONNECTTIMEOUT_MS as CONNECTTIMEOUT_MS;
pub use curl_ffi::CURLOPT_HTTP_TRANSFER_DECODING as HTTP_TRANSFER_DECODING;
pub use curl_ffi::CURLOPT_HTTP_CONTENT_DECODING as HTTP_CONTENT_DECODING;
pub use curl_ffi::CURLOPT_NEW_FILE_PERMS as NEW_FILE_PERMS;
pub use curl_ffi::CURLOPT_NEW_DIRECTORY_PERMS as NEW_DIRECTORY_PERMS;
pub use curl_ffi::CURLOPT_POSTREDIR as POSTREDIR;
pub use curl_ffi::CURLOPT_SSH_HOST_PUBLIC_KEY_MD5 as SSH_HOST_PUBLIC_KEY_MD5;
pub use curl_ffi::CURLOPT_OPENSOCKETFUNCTION as OPENSOCKETFUNCTION;
pub use curl_ffi::CURLOPT_OPENSOCKETDATA as OPENSOCKETDATA;
pub use curl_ffi::CURLOPT_COPYPOSTFIELDS as COPYPOSTFIELDS;
pub use curl_ffi::CURLOPT_PROXY_TRANSFER_MODE as PROXY_TRANSFER_MODE;
pub use curl_ffi::CURLOPT_SEEKFUNCTION as SEEKFUNCTION;
pub use curl_ffi::CURLOPT_SEEKDATA as SEEKDATA;
pub use curl_ffi::CURLOPT_CRLFILE as CRLFILE;
pub use curl_ffi::CURLOPT_ISSUERCERT as ISSUERCERT;
pub use curl_ffi::CURLOPT_ADDRESS_SCOPE as ADDRESS_SCOPE;
pub use curl_ffi::CURLOPT_CERTINFO as CERTINFO;
pub use curl_ffi::CURLOPT_USERNAME as USERNAME;
pub use curl_ffi::CURLOPT_PASSWORD as PASSWORD;
pub use curl_ffi::CURLOPT_PROXYUSERNAME as PROXYUSERNAME;
pub use curl_ffi::CURLOPT_PROXYPASSWORD as PROXYPASSWORD;
pub use curl_ffi::CURLOPT_NOPROXY as NOPROXY;
pub use curl_ffi::CURLOPT_TFTP_BLKSIZE as TFTP_BLKSIZE;
pub use curl_ffi::CURLOPT_SOCKS5_GSSAPI_SERVICE as SOCKS5_GSSAPI_SERVICE;
pub use curl_ffi::CURLOPT_SOCKS5_GSSAPI_NEC as SOCKS5_GSSAPI_NEC;
pub use curl_ffi::CURLOPT_PROTOCOLS as PROTOCOLS;
pub use curl_ffi::CURLOPT_REDIR_PROTOCOLS as REDIR_PROTOCOLS;
pub use curl_ffi::CURLOPT_SSH_KNOWNHOSTS as SSH_KNOWNHOSTS;
pub use curl_ffi::CURLOPT_SSH_KEYFUNCTION as SSH_KEYFUNCTION;
pub use curl_ffi::CURLOPT_SSH_KEYDATA as SSH_KEYDATA;
pub use curl_ffi::CURLOPT_MAIL_FROM as MAIL_FROM;
pub use curl_ffi::CURLOPT_MAIL_RCPT as MAIL_RCPT;
pub use curl_ffi::CURLOPT_FTP_USE_PRET as FTP_USE_PRET;
pub use curl_ffi::CURLOPT_RTSP_REQUEST as RTSP_REQUEST;
pub use curl_ffi::CURLOPT_RTSP_SESSION_ID as RTSP_SESSION_ID;
pub use curl_ffi::CURLOPT_RTSP_STREAM_URI as RTSP_STREAM_URI;
pub use curl_ffi::CURLOPT_RTSP_TRANSPORT as RTSP_TRANSPORT;
pub use curl_ffi::CURLOPT_RTSP_CLIENT_CSEQ as RTSP_CLIENT_CSEQ;
pub use curl_ffi::CURLOPT_RTSP_SERVER_CSEQ as RTSP_SERVER_CSEQ;
pub use curl_ffi::CURLOPT_INTERLEAVEDATA as INTERLEAVEDATA;
pub use curl_ffi::CURLOPT_INTERLEAVEFUNCTION as INTERLEAVEFUNCTION;
pub use curl_ffi::CURLOPT_WILDCARDMATCH as WILDCARDMATCH;
pub use curl_ffi::CURLOPT_CHUNK_BGN_FUNCTION as CHUNK_BGN_FUNCTION;
pub use curl_ffi::CURLOPT_CHUNK_END_FUNCTION as CHUNK_END_FUNCTION;
pub use curl_ffi::CURLOPT_FNMATCH_FUNCTION as FNMATCH_FUNCTION;
pub use curl_ffi::CURLOPT_CHUNK_DATA as CHUNK_DATA;
pub use curl_ffi::CURLOPT_FNMATCH_DATA as FNMATCH_DATA;
pub use curl_ffi::CURLOPT_RESOLVE as RESOLVE;
pub use curl_ffi::CURLOPT_TLSAUTH_USERNAME as TLSAUTH_USERNAME;
pub use curl_ffi::CURLOPT_TLSAUTH_PASSWORD as TLSAUTH_PASSWORD;
pub use curl_ffi::CURLOPT_TLSAUTH_TYPE as TLSAUTH_TYPE;
pub use curl_ffi::CURLOPT_TRANSFER_ENCODING as TRANSFER_ENCODING;
pub use curl_ffi::CURLOPT_CLOSESOCKETFUNCTION as CLOSESOCKETFUNCTION;
pub use curl_ffi::CURLOPT_CLOSESOCKETDATA as CLOSESOCKETDATA;
pub use curl_ffi::CURLOPT_GSSAPI_DELEGATION as GSSAPI_DELEGATION;
pub use curl_ffi::CURLOPT_DNS_SERVERS as DNS_SERVERS;
pub use curl_ffi::CURLOPT_ACCEPTTIMEOUT_MS as ACCEPTTIMEOUT_MS;
pub use curl_ffi::CURLOPT_TCP_KEEPALIVE as TCP_KEEPALIVE;
pub use curl_ffi::CURLOPT_TCP_KEEPIDLE as TCP_KEEPIDLE;
pub use curl_ffi::CURLOPT_TCP_KEEPINTVL as TCP_KEEPINTVL;
pub use curl_ffi::CURLOPT_SSL_OPTIONS as SSL_OPTIONS;
pub use curl_ffi::CURLOPT_MAIL_AUTH as MAIL_AUTH;
pub use curl_ffi::CURLOPT_SASL_IR as SASL_IR;
pub use curl_ffi::CURLOPT_XFERINFOFUNCTION as XFERINFOFUNCTION;
pub use curl_ffi::CURLOPT_XOAUTH2_BEARER as XOAUTH2_BEARER;
pub use curl_ffi::CURLOPT_DNS_INTERFACE as DNS_INTERFACE;
pub use curl_ffi::CURLOPT_DNS_LOCAL_IP4 as DNS_LOCAL_IP4;
pub use curl_ffi::CURLOPT_DNS_LOCAL_IP6 as DNS_LOCAL_IP6;
pub use curl_ffi::CURLOPT_LOGIN_OPTIONS as LOGIN_OPTIONS;
pub use curl_ffi::CURLOPT_SSL_ENABLE_NPN as SSL_ENABLE_NPN;
pub use curl_ffi::CURLOPT_SSL_ENABLE_ALPN as SSL_ENABLE_ALPN;
pub use curl_ffi::CURLOPT_EXPECT_100_TIMEOUT_MS as EXPECT_100_TIMEOUT_MS;
pub use curl_ffi::CURLOPT_PROXYHEADER as PROXYHEADER;
pub use curl_ffi::CURLOPT_HEADEROPT as HEADEROPT;
pub use curl_ffi::CURLOPT_POST301 as POST301;
pub use curl_ffi::CURLOPT_SSLKEYPASSWD as SSLKEYPASSWD;
pub use curl_ffi::CURLOPT_FTPAPPEND as FTPAPPEND;
pub use curl_ffi::CURLOPT_FTPLISTONLY as FTPLISTONLY;
pub use curl_ffi::CURLOPT_FTP_SSL as FTP_SSL;
pub use curl_ffi::CURLOPT_SSLCERTPASSWD as SSLCERTPASSWD;
pub use curl_ffi::CURLOPT_KRB4LEVEL as KRB4LEVEL;
pub use curl_ffi::CURLOPT_READDATA as READDATA;
pub use curl_ffi::CURLOPT_WRITEDATA as WRITEDATA;
pub use curl_ffi::CURLOPT_HEADERDATA as HEADERDATA;
pub use curl_ffi::CURLOPT_XFERINFODATA as XFERINFODATA;

pub type Opt = ffi::CURLoption;

pub trait OptVal {
    fn with_c_repr<F>(self, f: F) where F: FnOnce(*const c_void);
}

impl OptVal for isize {
    fn with_c_repr<F>(self, f: F) where F: FnOnce(*const c_void) {
        f(self as *const c_void)
    }
}

impl OptVal for i32 {
    fn with_c_repr<F>(self, f: F) where F: FnOnce(*const c_void) {
        (self as isize).with_c_repr(f)
    }
}

impl OptVal for usize {
    fn with_c_repr<F>(self, f: F) where F: FnOnce(*const c_void) {
        f(self as *const c_void)
    }
}

impl OptVal for bool {
    fn with_c_repr<F>(self, f: F) where F: FnOnce(*const c_void) {
        f(self as usize as *const c_void)
    }
}

impl<'a> OptVal for &'a str {
    fn with_c_repr<F>(self, f: F) where F: FnOnce(*const c_void) {
        let s = CString::new(self).unwrap();
        f(s.as_ptr() as *const c_void)
    }
}

impl<'a> OptVal for &'a Path {
    #[cfg(unix)]
    fn with_c_repr<F>(self, f: F) where F: FnOnce(*const c_void) {
        use std::ffi::OsStr;
        use std::os::unix::prelude::*;
        let s: &OsStr = self.as_ref();
        let s = CString::new(s.as_bytes()).unwrap();
        f(s.as_ptr() as *const c_void)
    }
    #[cfg(windows)]
    fn with_c_repr<F>(self, f: F) where F: FnOnce(*const c_void) {
        let s = CString::new(self.to_str().unwrap()).unwrap();
        f(s.as_ptr() as *const c_void)
    }
}
