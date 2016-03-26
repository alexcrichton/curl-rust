use curl::http::handle;
use super::server;

#[test]
pub fn test_simple_head() {
    let srv = server!(
        recv!(b"HEAD / HTTP/1.1\r\n\
                Host: localhost:{PORT}\r\n\
                Accept: */*\r\n\
                \r\n"), // Send the data
        send!(b"HTTP/1.1 200 OK\r\n\
                Content-Length: 5\r\n\r\n")
    );

    let res = handle()
        .head(server::url("/"))
        .exec();

    srv.assert();
    let res = res.unwrap();

    assert!(res.get_code() == 200, "code is {}", res.get_code());
    assert!(res.get_body().len() == 0);
    assert!(res.get_headers().len() == 1);
    assert!(res.get_header("content-length") == ["5".to_string()]);
}
