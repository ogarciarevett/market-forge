//! [`VenueSpec`] — the validated set of design choices that drives generation.

use crate::error::CodegenError;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Matching algorithm choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Matching {
    /// Price-time priority (classic CLOB).
    Fifo,
    /// Size-proportional allocation at the best level.
    ProRata,
}

/// Book data-structure choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum BookKind {
    /// `BTreeMap` per side — unbounded, idiomatic.
    BTreeMap,
    /// Bounded-tick bitmap book.
    Bitmap,
}

/// Concurrency-model choice.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum Concurrency {
    /// Apply orders inline on the calling thread.
    SingleThread,
    /// LMAX/DMAX-style pre-allocated ring buffer.
    Disruptor,
}

macro_rules! impl_choice {
    ($t:ty, $dim:literal, $expected:literal, { $($s:literal => $v:expr),+ $(,)? }) => {
        impl FromStr for $t {
            type Err = CodegenError;
            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s.trim().to_ascii_lowercase().as_str() {
                    $($s => Ok($v),)+
                    other => Err(CodegenError::InvalidChoice {
                        dimension: $dim,
                        value: other.to_string(),
                        expected: $expected,
                    }),
                }
            }
        }
    };
}

impl_choice!(Matching, "matching", "fifo | pro-rata", {
    "fifo" => Matching::Fifo,
    "pro-rata" => Matching::ProRata,
    "prorata" => Matching::ProRata,
});
impl_choice!(BookKind, "book", "btreemap | bitmap", {
    "btreemap" => BookKind::BTreeMap,
    "btree" => BookKind::BTreeMap,
    "bitmap" => BookKind::Bitmap,
});
impl_choice!(Concurrency, "concurrency", "single-thread | disruptor", {
    "single-thread" => Concurrency::SingleThread,
    "single" => Concurrency::SingleThread,
    "singlethread" => Concurrency::SingleThread,
    "disruptor" => Concurrency::Disruptor,
});

/// A complete, validated set of venue design choices.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct VenueSpec {
    /// The venue / crate name (lowercase, hyphenated).
    pub name: String,
    pub matching: Matching,
    pub book: BookKind,
    pub concurrency: Concurrency,
}

impl VenueSpec {
    /// Validate the venue name (a valid Cargo package name fragment).
    pub fn validate_name(name: &str) -> Result<(), CodegenError> {
        let ok = !name.is_empty()
            && name
                .bytes()
                .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-')
            && name.bytes().next().is_some_and(|b| b.is_ascii_lowercase());
        if ok {
            Ok(())
        } else {
            Err(CodegenError::InvalidName(name.to_string()))
        }
    }

    /// The Rust identifier form of the name (`my-clob` → `my_clob`).
    #[must_use]
    pub fn crate_ident(&self) -> String {
        self.name.replace('-', "_")
    }
}
