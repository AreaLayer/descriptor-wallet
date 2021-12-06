// Descriptor wallet library extending bitcoin & miniscript functionality
// by LNP/BP Association (https://lnp-bp.org)
// Written in 2020-2021 by
//     Dr. Maxim Orlovsky <orlovsky@pandoracore.com>
//
// To the extent possible under law, the author(s) have dedicated all
// copyright and related and neighboring rights to this software to
// the public domain worldwide. This software is distributed without
// any warranty.
//
// You should have received a copy of the Apache-2.0 License
// along with this software.
// If not, see <https://opensource.org/licenses/Apache-2.0>.

//! Library with extended support of hierarchival deterministic wallet
//! functions.
//!
//! Includes advanced derivation paths functionality and operations.

// Coding conventions
#![recursion_limit = "256"]
#![deny(dead_code, missing_docs, warnings)]

#[macro_use]
extern crate amplify;
#[macro_use]
extern crate strict_encoding;

#[cfg(feature = "serde")]
#[macro_use]
extern crate serde_crate as serde;

mod account;
mod derive;
mod indexes;
mod path;
mod ranges;
pub mod schemata;
mod traits;
mod xpubref;

pub use account::TrackingAccount;
pub use derive::{DeriveError, DerivePatternError, DerivePublicKey, DescriptorDerive};
pub use indexes::{AccountStep, HardenedIndex, SegmentIndexes, TerminalStep, UnhardenedIndex};
pub use path::DerivationSubpath;
pub use ranges::{IndexRange, IndexRangeList};
pub use schemata::DerivationScheme;
pub use traits::{DerivationPathMaster, HardenedNormalSplit};
pub use xpubref::XpubRef;

/// Constant determining BIP32 boundary for u32 values after which index
/// is treated as hardened
pub const HARDENED_INDEX_BOUNDARY: u32 = 1 << 31;

// TODO: Replace bip32::Error with more efficient local error type
