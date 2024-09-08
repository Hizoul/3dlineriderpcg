#[cfg(feature = "rng")]
pub mod rng;
#[cfg(feature = "rng")]
pub use rng::*;

#[cfg(feature = "env")]
pub mod env;
#[cfg(feature = "env")]
pub use env::*;

#[cfg(feature = "fs")]
pub mod fs;
#[cfg(feature = "fs")]
pub use fs::*;

#[cfg(feature = "http")]
pub mod http;
#[cfg(feature = "http")]
pub use http::*;

#[cfg(feature = "id")]
pub mod id;
#[cfg(feature = "id")]
pub use id::*;

#[cfg(target_arch = "wasm32")]
pub mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
#[cfg(not(target_arch = "wasm32"))]
pub mod desktop;
#[cfg(not(target_arch = "wasm32"))]
pub use desktop::*;
