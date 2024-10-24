mod error;

pub mod mint;
pub mod protocol;
pub mod testing;
pub mod wallet;

pub use self::{
    error::Error,
    protocol::{Decode, DecodeFields, Encode, EncodeFields},
};

#[macro_export]
macro_rules! unwind_panic {
    ($e:expr) => {
        match std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| $e)) {
            Ok(result) => result,
            Err(panic) => {
                let panic_msg = if let Some(s) = panic.downcast_ref::<String>() {
                    s.clone()
                } else if let Some(s) = panic.downcast_ref::<&str>() {
                    s.to_string()
                } else {
                    "Unknown panic".to_string()
                };

                return Err(Error::Panic(panic_msg));
            }
        }
    };
}
