pub use env::remove_var as remove;
pub use env::set_var as set;
pub use env::var as get;
use std::env;

pub const CONFIG_PATH: &str = "STORM_CONFIG_PATH";
