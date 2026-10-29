pub type Error = Box<dyn std::error::Error + Send + Sync + 'static>;
pub type Result<T, E = Error> = std::result::Result<T, E>;

#[macro_export]
macro_rules! _anyhow {
    ($msg:literal $(,)?) => {
        Box::<dyn std::error::Error + Send + Sync>::from($msg)
    };
    ($fmt:expr, $($arg:tt)*) => {
        Box::<dyn std::error::Error + Send + Sync>::from(format!($fmt, $($arg)*))
    };
}

#[macro_export]
macro_rules! _bail {
    ($($arg:tt)*) => {
        return Err($crate::_anyhow!($($arg)*))
    };
}

pub use _anyhow as anyhow;
#[allow(unused_imports)]
pub use _bail as bail;
