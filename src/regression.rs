use super::*;
use std::ffi::CString;

#[test]
fn short_dns_buffer_is_empty_and_bounded() {
    let host = CString::new("localhost").unwrap();
    let mut buffer = [b'X' as c_char; 16];
    unsafe {
        assert_eq!(dn_lookup_host(host.as_ptr(), true, buffer.as_mut_ptr(), 4), -1);
        assert_eq!(buffer[0], 0);
        assert_eq!(buffer[4], b'X' as c_char);
        assert_eq!(dn_lookup_host(host.as_ptr(), true, buffer.as_mut_ptr(), 0), -1);
    }
}

#[test]
fn invalid_utf8_returns_argument_error() {
    let bad = [0xffu8, 0];
    let bad = bad.as_ptr() as *const c_char;
    let mut buffer = [0 as c_char; 64];
    let mut timestamp = 123;
    unsafe {
        assert_eq!(dn_lookup_host(bad, true, buffer.as_mut_ptr(), buffer.len()), -1);
        assert_eq!(dn_connection_health(bad, 123, 100), -1);
        assert_eq!(dn_sntp_request(bad, 100, &mut timestamp), -1);
        assert_eq!(timestamp, 0);
    }
}

#[test]
fn local_ipv4_resolves_with_terminator() {
    let host = CString::new("localhost").unwrap();
    let mut buffer = [b'X' as c_char; 16];
    unsafe {
        assert_eq!(dn_lookup_host(host.as_ptr(), true, buffer.as_mut_ptr(), 10), 0);
        assert_eq!(from_c_str!(buffer.as_ptr()).unwrap(), "127.0.0.1");
        assert_eq!(buffer[9], 0);
        assert_eq!(buffer[10], b'X' as c_char);
    }
}

#[test]
fn sntp_errors_are_classified_without_errno() {
    // Socket UDP mudo: o pedido SNTP so pode terminar por timeout.
    let silent = std::net::UdpSocket::bind("127.0.0.1:0").unwrap();
    let silent = CString::new(silent.local_addr().unwrap().to_string()).unwrap();
    let no_port = CString::new("localhost").unwrap();
    let no_host = CString::new(":123").unwrap();
    let mut timestamp = 123;
    unsafe {
        assert_eq!(dn_sntp_request(silent.as_ptr(), 100, &mut timestamp), -2);
        assert_eq!(timestamp, 0);
        assert_eq!(dn_sntp_request(no_port.as_ptr(), 100, &mut timestamp), -3);
        assert_eq!(dn_sntp_request(no_host.as_ptr(), 100, &mut timestamp), -3);
    }
}

#[test]
fn connection_health_accepts_bare_and_bracketed_ipv6() {
    let listener = match std::net::TcpListener::bind("[::1]:0") {
        Ok(listener) => listener,
        Err(_) => return, // host sem IPv6 de loopback
    };
    let port = listener.local_addr().unwrap().port();
    for ip in ["::1", "[::1]"].iter() {
        let ip = CString::new(*ip).unwrap();
        assert_eq!(unsafe { dn_connection_health(ip.as_ptr(), port, 1000) }, 0);
    }
    let invalid = CString::new("not-an-ip").unwrap();
    assert_eq!(unsafe { dn_connection_health(invalid.as_ptr(), port, 1000) }, -4);
}
