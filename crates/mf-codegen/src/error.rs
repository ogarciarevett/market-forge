//! Error type for code generation.

/// Errors raised while validating a [`crate::VenueSpec`] or rendering a venue.
#[derive(Debug, thiserror::Error)]
pub enum CodegenError {
    /// A choice string did not match a known variant.
    #[error("invalid {dimension} choice {value:?} (expected one of: {expected})")]
    InvalidChoice {
        dimension: &'static str,
        value: String,
        expected: &'static str,
    },

    /// The chosen combination of algorithms cannot coherently compose.
    #[error("incoherent combination: {0}")]
    Incoherent(String),

    /// The venue name was empty or not a valid crate name.
    #[error("invalid venue name {0:?}: use lowercase letters, digits and hyphens")]
    InvalidName(String),

    /// Template rendering failed.
    #[error("template render failed: {0}")]
    Template(#[from] tera::Error),

    /// Writing the generated tree failed.
    #[error("filesystem error at {path}: {source}")]
    Io {
        path: String,
        #[source]
        source: std::io::Error,
    },
}
