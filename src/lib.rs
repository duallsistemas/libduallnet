use libc::{c_char, c_int, c_uint, size_t};
use mac_address::get_mac_address;
use serde_plain;
use sntpc;

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
    match dns_lookup::lookup_host(cs!(hostname).unwrap()) {
        Ok(ips) => {
            for item in ips {
                if prefer_ipv4 && !item.is_ipv4() {
                    continue;
                }
                let addr = sc!(to_string!(item).unwrap()).unwrap();
                let buf = addr.to_bytes_with_nul();
                let mut buf_size = size;
                if buf_size > buf.len() {
                    buf_size = buf.len()
                }
                copy!(buf.as_ptr(), ip, buf_size);
                return 0;
            }
            return -2;
        }
        Err(_) => return -3,
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
            let addr = sc!(format!("{}", ma)).unwrap();
            let buf = addr.to_bytes_with_nul();
            let mut buf_size = size;
            if buf_size > buf.len() {
                buf_size = buf.len()
            }
            copy!(buf.as_ptr(), mac_addr, buf_size);
            return 0;
        }
        Ok(None) => return -2,
        Err(_) => return -3,
    }
}

/// Requests timestamp from a given NTP server.
///
/// # Arguments
///
/// * `[in] pool` - Server's name or IP address as C-like string.
/// * `[in] port` - Server's port.
/// * `[in,out] size` - Returned timestamp.
///
/// # Returns
///
/// * `0` - Success.
/// * `-1` - Invalid argument.
/// * `-2` - NTP error.
#[no_mangle]
pub unsafe extern "C" fn dn_ntp_request(
    pool: *const c_char,
    port: c_uint,
    timestamp: *mut c_uint,
) -> c_int {
    if pool.is_null() || port <= 0 || timestamp.is_null() {
        return -1;
    }
    let result = sntpc::request(cs!(pool).unwrap(), port);
    match result {
        Ok(time) => {
            *timestamp = time;
            return 0;
        }
        Err(_) => return -2,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn version() {
        unsafe {
            assert_eq!(cs!(dn_version()).unwrap(), env!("CARGO_PKG_VERSION"));
        }
    }

    #[test]
    fn lookup_host() {
        unsafe {
            let ip: [c_char; 45] = [0; 45];
            assert_eq!(
                dn_lookup_host(std::ptr::null(), true, ip.as_ptr() as *mut c_char, ip.len()),
                -1
            );
            assert_eq!(
                dn_lookup_host(
                    sc!("::1").unwrap().as_ptr(),
                    true,
                    std::ptr::null_mut(),
                    ip.len()
                ),
                -1
            );
            assert_eq!(
                dn_lookup_host(
                    sc!("::1").unwrap().as_ptr(),
                    true,
                    ip.as_ptr() as *mut c_char,
                    0
                ),
                -1
            );
            assert_eq!(
                dn_lookup_host(
                    sc!("::1").unwrap().as_ptr(),
                    true,
                    ip.as_ptr() as *mut c_char,
                    ip.len()
                ),
                -2
            );
            assert_eq!(
                dn_lookup_host(
                    sc!("abc123").unwrap().as_ptr(),
                    false,
                    ip.as_ptr() as *mut c_char,
                    ip.len()
                ),
                -3
            );

            dn_lookup_host(
                sc!("localhost").unwrap().as_ptr(),
                true,
                ip.as_ptr() as *mut c_char,
                ip.len(),
            );
            let len = len!(ip.as_ptr());
            assert_eq!(len, 9);
            assert_eq!(
                cmp!(ip.as_ptr(), sc!("127.0.0.1").unwrap().as_ptr(), len + 1),
                0
            );
            dn_lookup_host(
                sc!("localhost").unwrap().as_ptr(),
                false,
                ip.as_ptr() as *mut c_char,
                ip.len(),
            );
            let len = len!(ip.as_ptr());
            assert_eq!(len, 3);
            assert_eq!(cmp!(ip.as_ptr(), sc!("::1").unwrap().as_ptr(), len + 1), 0);
        }
    }

    #[test]
    fn mac_address() {
        unsafe {
            let mac_addr: [c_char; 18] = [0; 18];
            assert_eq!(dn_mac_address(std::ptr::null_mut(), mac_addr.len()), -1);
            assert_eq!(dn_mac_address(mac_addr.as_ptr() as *mut c_char, 0), -1);

            dn_mac_address(mac_addr.as_ptr() as *mut c_char, mac_addr.len());
            let len = len!(mac_addr.as_ptr());
            assert_eq!(len, 17);
            let mac = format!("{}", get_mac_address().unwrap().unwrap());
            assert_eq!(
                cmp!(mac_addr.as_ptr(), sc!(mac).unwrap().as_ptr(), len + 1),
                0
            );
        }
    }

    #[test]
    fn ntp_request() {
        unsafe {
            let pool = sc!("pool.ntp.org").unwrap().as_ptr();
            let mut timestamp: c_uint = 0;
            assert_eq!(
                dn_ntp_request(std::ptr::null_mut(), 123, &mut timestamp),
                -1
            );
            assert_eq!(dn_ntp_request(pool, 0, &mut timestamp), -1);
            assert_eq!(dn_ntp_request(pool, 123, std::ptr::null_mut()), -1);
            assert_eq!(dn_ntp_request(pool, 321, &mut timestamp), -2);

            assert_eq!(
                dn_ntp_request(sc!("pool.ntp.org").unwrap().as_ptr(), 123, &mut timestamp),
                0
            );
        }
    }
}
