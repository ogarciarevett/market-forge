//! `mf-codegen` — turn a [`VenueSpec`] into a generated Cargo workspace.
//!
//! Three pieces: the [`VenueSpec`] (validated design choices), the compatibility
//! [`matrix`] (rejects incoherent combinations), and [`render_venue`] (Tera templates → tree).

#![forbid(unsafe_code)]

mod error;
pub mod matrix;
mod render;
mod spec;

pub use error::CodegenError;
pub use render::render_venue;
pub use spec::{BookKind, Concurrency, Matching, VenueSpec};
