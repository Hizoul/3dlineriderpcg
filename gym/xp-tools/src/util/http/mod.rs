#[cfg(target_arch = "wasm32")]
pub mod wasm;
#[cfg(target_arch = "wasm32")]
pub use wasm::*;
#[cfg(not(target_arch = "wasm32"))]
pub mod desktop;
#[cfg(not(target_arch = "wasm32"))]
pub use desktop::*;

#[cfg(feature = "ipfs")]
pub mod ipfs;
#[cfg(feature = "ipfs")]
pub use ipfs::*;