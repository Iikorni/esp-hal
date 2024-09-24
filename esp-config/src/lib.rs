#![doc = include_str!("../README.md")]
//! ## Feature Flags
#![doc = document_features::document_features!(feature_label = r#"<span class="stab portability"><code>{feature}</code></span>"#)]
#![doc(html_logo_url = "https://avatars.githubusercontent.com/u/46717278")]
#![cfg_attr(not(feature = "build"), no_std)]

#[cfg(feature = "build")]
mod generate;
#[cfg(feature = "build")]
pub use generate::*;

#[macro_export]
// TODO from 1.82 we can use <$ty>::from_str_radix(env!($var), 10) instead
/// Parse the value of a `env` variable as an integer at compile time
macro_rules! esp_config_int {
    ($ty:ty, $var:expr) => {
        const {
            const BYTES: &[u8] = env!($var).as_bytes();
            esp_config_int_parse!($ty, BYTES)
        }
    };
}

#[macro_export]
/// Get the string value of an `env` variable at compile time
macro_rules! esp_config_str {
    ($var:expr) => {
        env!($var)
    };
}

#[macro_export]
/// Parse the value of a `env` variable as an bool at compile time
macro_rules! esp_config_bool {
    ($var:expr) => {
        match env!($var).as_bytes() {
            b"false" => false,
            _ => true,
        }
    };
}

#[macro_export]
#[doc(hidden)] // to avoid confusion with esp_config_int, let's hide this
/// Parse a string like "777" into an integer, which _can_ be used in a `const`
/// context
macro_rules! esp_config_int_parse {
    ($ty:ty, $bytes:expr) => {{
        let mut bytes = $bytes;
        let mut val: $ty = 0;
        let mut sign_seen = false;
        let mut is_negative = false;
        while let [byte, rest @ ..] = bytes {
            match *byte {
                b'0'..=b'9' => {
                    val = val * 10 + (*byte - b'0') as $ty;
                }
                b'-' | b'+' if !sign_seen => {
                    if *byte == b'-' {
                        is_negative = true;
                    }
                    sign_seen = true;
                }
                _ => ::core::panic!("invalid digit"),
            }
            bytes = rest;
        }
        if is_negative {
            let original = val;
            // subtract twice to get the negative
            val -= original;
            val -= original;
        }
        val
    }};
}

#[cfg(test)]
mod test {

    // We can only test success in the const context
    const _: () = {
        core::assert!(esp_config_int_parse!(i64, "-77777".as_bytes()) == -77777);
        core::assert!(esp_config_int_parse!(isize, "-7777".as_bytes()) == -7777);
        core::assert!(esp_config_int_parse!(i32, "-999".as_bytes()) == -999);
        core::assert!(esp_config_int_parse!(i16, "-99".as_bytes()) == -99);
        core::assert!(esp_config_int_parse!(i8, "-9".as_bytes()) == -9);

        core::assert!(esp_config_int_parse!(u64, "77777".as_bytes()) == 77777);
        core::assert!(esp_config_int_parse!(usize, "7777".as_bytes()) == 7777);
        core::assert!(esp_config_int_parse!(u32, "999".as_bytes()) == 999);
        core::assert!(esp_config_int_parse!(u16, "99".as_bytes()) == 99);
        core::assert!(esp_config_int_parse!(u8, "9".as_bytes()) == 9);
    };

    #[test]
    #[should_panic]
    fn test_expect_positive() {
        esp_config_int_parse!(u8, "-5".as_bytes());
    }
}