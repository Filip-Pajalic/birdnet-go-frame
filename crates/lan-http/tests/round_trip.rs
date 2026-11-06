//! The client against the server, over real sockets.

#![allow(clippy::expect_used)] // Tests may panic on setup failures.

use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;
use std::time::Duration;

use lan_http::{Response, Server, get};

fn serve(handler: fn(&lan_http::Request) -> Response) -> String {
    let server = Server::bind("127.0.0.1:0").expect("bind");
    let address = server.local_addr().expect("address");
    thread::spawn(move || server.run(handler));
    format!("http://{address}")
}

#[test]
fn get_round_trip_with_headers_and_status() {
    let base = serve(|request| match request.path() {
        "/hello" => Response::text(200, request.target()).with_header(
            "ETag",
            request.header("if-none-match").unwrap_or("none").to_owned(),
        ),
        _ => Response::text(404, "not found"),
    });
    let response = get(&format!("{base}/hello?x=1"), Duration::from_secs(5)).expect("fetch");
    assert_eq!(response.status, 200);
    assert_eq!(response.body, b"/hello?x=1");
    let missing = get(&format!("{base}/other"), Duration::from_secs(5)).expect("fetch");
    assert_eq!(missing.status, 404);
}

#[test]
fn large_bodies_arrive_whole() {
    let base = serve(|_| Response::new(200, vec![7_u8; 1_000_000]));
    let response = get(&base, Duration::from_secs(5)).expect("fetch");
    assert_eq!(response.body.len(), 1_000_000);
}

#[test]
fn other_methods_and_garbage_are_refused() {
    let base = serve(|_| Response::text(200, "ok"));
    let address = base.trim_start_matches("http://").to_owned();
    let exchange = |request: &[u8]| -> String {
        let mut stream = TcpStream::connect(&address).expect("connect");
        stream.write_all(request).expect("send");
        let mut reply = String::new();
        stream.read_to_string(&mut reply).expect("read");
        reply
    };
    assert!(exchange(b"POST / HTTP/1.1\r\n\r\n").starts_with("HTTP/1.1 405"));
    assert!(exchange(b"hello\r\n\r\n").starts_with("HTTP/1.1 400"));
    let head = exchange(b"HEAD / HTTP/1.1\r\n\r\n");
    assert!(
        head.starts_with("HTTP/1.1 200")
            && head.contains("Content-Length: 2")
            && !head.ends_with("ok")
    );
}

#[test]
fn reads_chunked_responses_from_other_servers() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let address = listener.local_addr().expect("address");
    thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        let mut request = [0_u8; 1024];
        let _ = stream.read(&mut request).expect("read");
        stream
            .write_all(b"HTTP/1.1 200 OK\r\nTransfer-Encoding: chunked\r\n\r\n2\r\n[]\r\n0\r\n\r\n")
            .expect("write");
    });
    let response = get(&format!("http://{address}/api"), Duration::from_secs(5)).expect("fetch");
    assert_eq!(response.body, b"[]");
}

#[test]
fn unreachable_servers_fail_quickly() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let address = listener.local_addr().expect("address");
    drop(listener);
    assert!(get(&format!("http://{address}/"), Duration::from_secs(2)).is_err());
}

#[test]
fn posts_a_body_with_headers_and_reads_response_headers() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind");
    let address = listener.local_addr().expect("address");
    let seen = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept");
        // TCP may split the request anywhere: read until the whole body is in.
        let mut request: Vec<u8> = Vec::new();
        let mut chunk = [0_u8; 4096];
        while !request.ends_with(br#"{"password":"pw"}"#) {
            let read = stream.read(&mut chunk).expect("read");
            assert!(read > 0, "the request ended early");
            request.extend_from_slice(&chunk[..read]);
        }
        stream
            .write_all(
                b"HTTP/1.1 200 OK\r\nSet-Cookie: a=1; Path=/\r\nSet-Cookie: b=2\r\n\
                  Content-Length: 2\r\n\r\nok",
            )
            .expect("write");
        String::from_utf8_lossy(&request).into_owned()
    });
    let response = lan_http::send(
        "POST",
        &format!("http://{address}/api/v2/auth/login"),
        &[("Content-Type", "application/json"), ("Cookie", "s=x")],
        br#"{"password":"pw"}"#,
        Duration::from_secs(5),
    )
    .expect("send");
    let request: String = seen.join().expect("server thread");
    assert!(
        request.starts_with("POST /api/v2/auth/login HTTP/1.1\r\n"),
        "{request}"
    );
    assert!(request.contains("Content-Type: application/json\r\n"));
    assert!(request.contains("Cookie: s=x\r\n"));
    assert!(request.contains("Content-Length: 17\r\n"));
    assert!(request.ends_with("\r\n\r\n{\"password\":\"pw\"}"));
    let cookies: Vec<&str> = response
        .headers
        .iter()
        .filter(|(name, _)| name.eq_ignore_ascii_case("set-cookie"))
        .map(|(_, value)| value.as_str())
        .collect();
    assert_eq!(cookies, ["a=1; Path=/", "b=2"]);
    assert!(
        lan_http::send(
            "GET",
            "http://127.0.0.1:1/",
            &[("X", "a\r\nb")],
            &[],
            Duration::from_secs(1)
        )
        .is_err()
    );
}
