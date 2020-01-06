use std::io::{Error, ErrorKind::TimedOut};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::ptr;
use std::time::Duration;

use libc::{c_char, c_int, size_t};
use mac_address::get_mac_address;
use sntp_request::SntpRequest;

mod utils;

/// Retrieves library version as C-like string.
#[no_mangle]
pub unsafe extern "C" fn dn_version() -> *const c_char {
    concat!(env!("CARGO_PKG_VERSION"), '\0').as_ptr() as *const c_char
}

/// Lookups the address for a given hostname via DNS.
///
/// # Arguments
///
/// * `[in] hostname` - Hostname as C-like string.
/// * `[in] prefer_ipv4` - Prefer to return IPv4 address.
/// * `[in,out] ip` - Resolved address as IPv4 or IPv6 into C-like string.
/// * `[in] size` - Size of the `hostname` string.
///
/// # Returns
///
/// * `0` - Success.
/// * `-1` - Invalid argument.
/// * `-2` - No address found.
/// * `-3` - Unknown error.
#[no_mangle]
pub unsafe extern "C" fn dn_lookup_host(
    hostname: *const c_char,
    prefer_ipv4: bool,
    ip: *mut c_char,
    size: size_t,
) -> c_int {
    if hostname.is_null() || ip.is_null() || size <= 0 {
        return -1;
    }
    match vec![from_c_str!(hostname).unwrap(), "0"]
        .join(":")
        .to_socket_addrs()
    {
        Ok(addrs) => {
            for addr in addrs {
                if prefer_ipv4 && !addr.is_ipv4() {
                    continue;
                }
                let mut resolved_ip = addr.to_string();
                resolved_ip.truncate(resolved_ip.len() - ":0".len());
                let dest_ip = to_c_str!(resolved_ip).unwrap();
                copy_c_str!(dest_ip, ip, size);
                return 0;
            }
            -2
        }
        Err(_) => -3,
    }
}

/// Checks a TCP connection to a remote host with a timeout.
///
/// # Arguments
///
/// * `[in] ip` - IP address as C-like string.
/// * `[in] port` - Connection port.
/// * `[in] timeout` - Connection timeout in milliseconds.
///
/// # Returns
///
/// * `0` - Success.
/// * `-1` - Invalid argument.
/// * `-2` - Connection timed out.
/// * `-3` - Could not connect.
/// * `-4` - Unknown error.
#[no_mangle]
pub unsafe extern "C" fn dn_connection_health(ip: *const c_char, port: u16, timeout: u64) -> c_int {
    if ip.is_null() || port <= 0 {
        return -1;
    }
    match format!("{}:{}", from_c_str!(ip).unwrap(), port).parse::<SocketAddr>() {
        Ok(addr) => match TcpStream::connect_timeout(&addr, Duration::from_millis(timeout)) {
            Ok(_) => 0,
            Err(error) => {
                if error.kind() == TimedOut {
                    return -2;
                }
                -3
            }
        },
        Err(_) => -4,
    }
}

/// Retrieves the MAC address of the first active network device.
///
/// # Arguments
///
/// * `[in,out] mac_addr` - MAC address as C-like string.
/// * `[in] size` - Size of the `mac_addr` string.
///
/// # Returns
///
/// * `0` - Success.
/// * `-1` - Invalid argument.
/// * `-2` - No MAC address found.
/// * `-3` - Unknown error.
#[no_mangle]
pub unsafe extern "C" fn dn_mac_address(mac_addr: *mut c_char, size: size_t) -> c_int {
    if mac_addr.is_null() || size <= 0 {
        return -1;
    }
    match get_mac_address() {
        Ok(Some(ma)) => {
            let addr = to_c_str!(ma.to_string()).unwrap();
            copy_c_str!(addr, mac_addr, size);
            0
        }
        Ok(None) => -2,
        Err(_) => -3,
    }
}

/// Requests timestamp from a given NTP server via SNTP protocol.
///
/// # Arguments
///
/// * `[in] addr` - Server address as C-like string, e.g.: pool.ntp.org:123.
/// * `[in] timeout` - Time out in milliseconds (default 5000).
/// * `[in,out] timestamp` - Returned timestamp.
///
/// # Returns
///
/// * `0` - Success.
/// * `-1` - Invalid argument.
/// * `-2` - Reached time out.
/// * `-3` - Unknown error.
#[no_mangle]
pub unsafe extern "C" fn dn_sntp_request(
    addr: *const c_char,
    timeout: u64,
    timestamp: *mut i64,
) -> c_int {
    if timestamp.is_null() {
        return -1;
    }
    let sntp = SntpRequest::new();
    if (timeout > 0) && !sntp.set_timeout(Duration::from_millis(timeout)).is_ok() {
        return -1;
    }
    let result: Result<i64, Error>;
    if addr.is_null() {
        result = sntp.get_unix_time();
    } else {
        result = sntp.get_unix_time_by_addr(from_c_str!(addr).unwrap());
    }
    match result {
        Ok(ts) => {
            *timestamp = ts;
            0
        }
        Err(error) => {
            if error.kind() == TimedOut {
                return -2;
            }
            -3
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::thread;

    #[test]
    fn version() {
        unsafe {
            assert_eq!(
                from_c_str!(dn_version()).unwrap(),
                env!("CARGO_PKG_VERSION")
            );
        }
    }

    #[test]
    fn lookup_host() {
        unsafe {
            let ip: [c_char; 45] = [0; 45];
            assert_eq!(
                dn_lookup_host(ptr::null(), true, ip.as_ptr() as *mut c_char, ip.len()),
                -1
            );
            assert_eq!(
                dn_lookup_host(
                    to_c_str!("::1").unwrap().as_ptr(),
                    true,
                    ptr::null_mut(),
                    ip.len()
                ),
                -1
            );
            assert_eq!(
                dn_lookup_host(
                    to_c_str!("::1").unwrap().as_ptr(),
                    true,
                    ip.as_ptr() as *mut c_char,
                    0
                ),
                -1
            );
            assert_eq!(
                dn_lookup_host(
                    to_c_str!("::1").unwrap().as_ptr(),
                    true,
                    ip.as_ptr() as *mut c_char,
                    ip.len()
                ),
                -2
            );
            assert_eq!(
                dn_lookup_host(
                    to_c_str!("abc123").unwrap().as_ptr(),
                    false,
                    ip.as_ptr() as *mut c_char,
                    ip.len()
                ),
                -3
            );

            assert_eq!(
                dn_lookup_host(
                    to_c_str!("localhost").unwrap().as_ptr(),
                    true,
                    ip.as_ptr() as *mut c_char,
                    ip.len(),
                ),
                0
            );
            let len = length!(ip.as_ptr());
            assert_eq!(len, 9);
            assert_eq!(
                compare!(
                    ip.as_ptr(),
                    to_c_str!("127.0.0.1").unwrap().as_ptr(),
                    len + 1
                ),
                0
            );
            assert_eq!(
                dn_lookup_host(
                    to_c_str!("localhost").unwrap().as_ptr(),
                    false,
                    ip.as_ptr() as *mut c_char,
                    ip.len(),
                ),
                0
            );
            let len = length!(ip.as_ptr());
            assert_eq!(len, 5);
            assert_eq!(
                compare!(ip.as_ptr(), to_c_str!("[::1]").unwrap().as_ptr(), len + 1),
                0
            );
        }
    }

    #[test]
    fn connection_health() {
        unsafe {
            assert_eq!(dn_connection_health(ptr::null_mut(), 123, 3000), -1);
            assert_eq!(
                dn_connection_health(to_c_str!("127.0.0.1").unwrap().as_ptr(), 0, 3000),
                -1
            );
            assert_eq!(
                dn_connection_health(to_c_str!("54.94.220.237").unwrap().as_ptr(), 443, 3000),
                0
            );
        }
    }

    #[test]
    fn mac_address() {
        unsafe {
            let mac_addr: [c_char; 18] = [0; 18];
            assert_eq!(dn_mac_address(ptr::null_mut(), mac_addr.len()), -1);
            assert_eq!(dn_mac_address(mac_addr.as_ptr() as *mut c_char, 0), -1);

            dn_mac_address(mac_addr.as_ptr() as *mut c_char, mac_addr.len());
            let len = length!(mac_addr.as_ptr());
            assert_eq!(len, 17);
            let mac = format!("{}", get_mac_address().unwrap().unwrap());
            assert_eq!(
                compare!(mac_addr.as_ptr(), to_c_str!(mac).unwrap().as_ptr(), len + 1),
                0
            );
        }
    }

    #[test]
    fn sntp_request() {
        unsafe {
            let addr = to_c_str!("pool.ntp.org:123").unwrap().as_ptr();
            let mut ts: i64 = 0;
            assert_eq!(dn_sntp_request(addr, 0, ptr::null_mut()), -1);
            assert_eq!(
                dn_sntp_request(
                    to_c_str!("pool.ntp.org:321").unwrap().as_ptr(),
                    100,
                    &mut ts
                ),
                -3
            );

            let mut ts1: i64 = 0;
            let mut ts2: i64 = 0;
            assert_eq!(dn_sntp_request(ptr::null(), 0, &mut ts1), 0);
            thread::sleep(Duration::from_secs(2));
            assert_eq!(dn_sntp_request(ptr::null(), 0, &mut ts2), 0);
            assert!(ts2 > ts1);
        }
    }
}
