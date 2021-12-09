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

use core::fmt::{self, Display, Formatter};
use core::str::FromStr;
use std::io;

use bitcoin::util::bip32;
use strict_encoding::{self, StrictDecode, StrictEncode};

use crate::SegmentIndexes;

/// Derivation path that consisting only of single type of segments.
///
/// Useful in specifying concrete derivation from a provided extended public key
/// without extended private key accessible.
///
/// Type guarantees that the number of derivation path segments is non-zero.
#[derive(Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, From)]
pub struct DerivationSubpath<Segment>(Vec<Segment>)
where
    Segment: SegmentIndexes;

impl<Segment> From<&[Segment]> for DerivationSubpath<Segment>
where
    Segment: SegmentIndexes,
{
    fn from(path: &[Segment]) -> Self { Self(path.to_vec()) }
}

impl<Segment> AsRef<[Segment]> for DerivationSubpath<Segment>
where
    Segment: SegmentIndexes,
{
    #[inline]
    fn as_ref(&self) -> &[Segment] { &self.0 }
}

impl<Segment> StrictEncode for DerivationSubpath<Segment>
where
    Segment: SegmentIndexes + StrictEncode,
{
    #[inline]
    fn strict_encode<E: io::Write>(&self, e: E) -> Result<usize, strict_encoding::Error> {
        self.0.strict_encode(e)
    }
}

impl<Segment> StrictDecode for DerivationSubpath<Segment>
where
    Segment: SegmentIndexes + StrictDecode,
{
    #[inline]
    fn strict_decode<D: io::Read>(d: D) -> Result<Self, strict_encoding::Error> {
        Ok(Self(Vec::strict_decode(d)?))
    }
}

impl<Segment> Display for DerivationSubpath<Segment>
where
    Segment: SegmentIndexes + Display,
{
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        for segment in &self.0 {
            f.write_str("/")?;
            Display::fmt(segment, f)?;
        }
        Ok(())
    }
}

impl<Segment> FromStr for DerivationSubpath<Segment>
where
    Segment: SegmentIndexes + FromStr,
    bip32::Error: From<<Segment as FromStr>::Err>,
{
    type Err = bip32::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with('/') {
            return Err(bip32::Error::InvalidDerivationPathFormat);
        }
        let inner = s[1..]
            .split('/')
            .map(Segment::from_str)
            .collect::<Result<Vec<_>, Segment::Err>>()?;
        if inner.is_empty() {
            return Err(bip32::Error::InvalidDerivationPathFormat);
        }
        Ok(Self(inner))
    }
}
