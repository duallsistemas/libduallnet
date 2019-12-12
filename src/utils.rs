#[doc(hidden)]
#[macro_export]
macro_rules! from_c_str {
    ($cstr:expr) => {
        std::ffi::CStr::from_ptr($cstr).to_str()
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! to_c_str {
    ($string:expr) => {
        std::ffi::CString::new($string)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! to_string {
    ($raw:expr) => {
        serde_plain::to_string(&$raw)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! copy {
    ($src:expr,$dest:expr,$size:expr) => {
        libc::memcpy(
            $dest as *mut libc::c_void,
            $src as *const libc::c_void,
            $size,
        )
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! compare {
    ($a:expr,$b:expr,$size:expr) => {
        libc::memcmp($a as *const libc::c_void, $b as *const libc::c_void, $size)
    };
}

#[doc(hidden)]
#[macro_export]
macro_rules! length {
    ($cstr:expr) => {
        libc::strlen($cstr)
    };
}
