// Descriptor wallet library extending bitcoin & miniscript functionality
// by LNP/BP Association (https://lnp-bp.org)
// Written in 2020-2022 by
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

//! Processing proprietary PSBT keys related to taproot-based OP_RETURN
//! (or tapret) commitments.
//!
//! NB: Wallets supporting tapret commitments must do that through the use of
//! deterministic bitcoin commitments crate (`bp-dpc`) in order to ensure
//! that multiple protocols can put commitment inside the same transaction
//! without collisions between them.
//!
//! This module provides support for marking PSBT outputs which may host
//! tapreturn commitment and populating PSBT with the data related to tapret
//! commitments.

use amplify::Slice32;
use bitcoin::util::taproot::TaprootMerkleBranch;

use crate::{OutputMap, ProprietaryKey};

/// PSBT proprietary key prefix used for tapreturn commitment.
pub const PSBT_TAPRET_PREFIX: &[u8] = b"TAPRET";
/// Proprietary key subtype marking PSBT outputs which may host tapreturn
/// commitment.
pub const PSBT_OUT_TAPRET_HOST: u8 = 0x08;
/// Proprietary key subtype holding 32-byte commitment which will be put into
/// tapreturn tweak.
pub const PSBT_OUT_TAPRET_COMMITMENT: u8 = 0x09;
/// Proprietary key subtype holding merkle branch path to tapreturn tweak inside
/// the taptree structure.
pub const PSBT_OUT_TAPRET_PROOF: u8 = 0x0a;

/// Extension trait for static functions returning tapreturn-related proprietary
/// keys.
pub trait ProprietaryKeyTapret {
    /// Constructs [`PSBT_OUT_TAPRET_HOST`] proprietary key.
    fn tapret_host() -> ProprietaryKey {
        ProprietaryKey {
            prefix: PSBT_TAPRET_PREFIX.to_vec(),
            subtype: PSBT_OUT_TAPRET_HOST,
            key: vec![],
        }
    }

    /// Constructs [`PSBT_OUT_TAPRET_COMMITMENT`] proprietary key.
    fn tapret_commitment() -> ProprietaryKey {
        ProprietaryKey {
            prefix: PSBT_TAPRET_PREFIX.to_vec(),
            subtype: PSBT_OUT_TAPRET_COMMITMENT,
            key: vec![],
        }
    }

    /// Constructs [`PSBT_OUT_TAPRET_PROOF`] proprietary key.
    fn tapret_proof() -> ProprietaryKey {
        ProprietaryKey {
            prefix: PSBT_TAPRET_PREFIX.to_vec(),
            subtype: PSBT_OUT_TAPRET_PROOF,
            key: vec![],
        }
    }
}

impl ProprietaryKeyTapret for ProprietaryKey {}

/// Errors processing tapret-related proprietary PSBT keys and their values.
#[derive(
    Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, Debug, Display, Error
)]
#[display(doc_comments)]
pub enum KeyError {
    /// output already contains commitment; there must be a single commitment
    /// per output.
    OutputAlreadyHasCommitment,
}

/// Extension trait adding support for tapreturn commitments to PSBT [`Output`].
pub trait TapretOutput {
    /// Returns whether this output may contain tapret commitment. This is
    /// detected by the presence of the empty [`PSBT_OUT_TAPRET_HOST`] key.
    fn can_host_tapret(&self) -> bool;

    /// Sets whether this output may contain tapret commitment bu adding or
    /// removing [`PSBT_OUT_TAPRET_HOST`] key basing on `can_host_commitment`
    /// value.
    fn set_can_host_tapret(&mut self, can_host_commitment: bool) -> bool;

    /// Detects presence of a vaid [`PSBT_OUT_TAPRET_COMMITMENT`].
    ///
    /// If [`PSBT_OUT_TAPRET_COMMITMENT`] is absent or its value is invalid,
    /// returns `false`. In the future, when `PSBT_OUT_TAPRET_COMMITMENT` will
    /// become a standard and non-custom key, PSBTs with invalid key values
    /// will error at deserialization and this function will return `false`
    /// only in cases when the output does not have
    /// `PSBT_OUT_TAPRET_COMMITMENT`.
    fn has_tapret_commitment(&self) -> bool;

    /// Returns valid tapret commitment from the [`PSBT_OUT_TAPRET_COMMITMENT`]
    /// key, if present. If the commitment is absent or invalid, returns
    /// `None`.
    ///
    /// We do not error on invalid commitments in order to support future update
    /// of this proprietary key to the standard one. In this case, the
    /// invalid commitments (having non-32 bytes) will be filtered at the
    /// moment of PSBT deserialization and this function will return `None`
    /// only in situations when the commitment is absent.
    fn tapret_commitment(&self) -> Option<Slice32>;

    /// Assigns value of the tapreturn commitment to this PSBT output, by
    /// adding [`PSBT_OUT_TAPRET_COMMITMENT`] proprietary key containing the
    /// 32-byte commitment as its value.
    ///
    /// Errors with [`KeyError::OutputAlreadyHasCommitment`] if the commitment
    /// is already present in the output.
    fn set_tapret_commitment(&mut self, commitment: impl Into<[u8; 32]>) -> Result<(), KeyError>;

    /// Detects presence of a valid [`PSBT_OUT_TAPRET_PROOF`].
    ///
    /// If [`PSBT_OUT_TAPRET_PROOF`] is absent or its value is invalid,
    /// returns `false`. In the future, when `PSBT_OUT_TAPRET_PROOF` will
    /// become a standard and non-custom key, PSBTs with invalid key values
    /// will error at deserialization and this function will return `false`
    /// only in cases when the output does not have `PSBT_OUT_TAPRET_PROOF`.
    fn has_tapret_proof(&self) -> bool;

    /// Returns valid tapret commitment proof from the [`PSBT_OUT_TAPRET_PROOF`]
    /// key, if present. If the commitment is absent or invalid, returns `None`.
    ///
    /// We do not error on invalid proofs in order to support future update of
    /// this proprietary key to the standard one. In this case, the invalid
    /// commitments (having non-32 bytes) will be filtered at the moment of PSBT
    /// deserialization and this function will return `None` only in situations
    /// when the commitment is absent.
    fn tapret_proof(&self) -> Option<TaprootMerkleBranch>;
}

impl TapretOutput for OutputMap {
    #[inline]
    fn can_host_tapret(&self) -> bool {
        self.proprietary
            .contains_key(&ProprietaryKey::tapret_host())
    }

    fn set_can_host_tapret(&mut self, can_host_commitment: bool) -> bool {
        let prev = self.can_host_tapret();
        if can_host_commitment {
            self.proprietary
                .insert(ProprietaryKey::tapret_host(), vec![]);
        } else {
            self.proprietary.remove(&ProprietaryKey::tapret_host());
        }
        prev
    }

    fn has_tapret_commitment(&self) -> bool {
        self.proprietary
            .contains_key(&ProprietaryKey::tapret_commitment())
    }

    fn tapret_commitment(&self) -> Option<Slice32> {
        self.proprietary
            .get(&ProprietaryKey::tapret_commitment())
            .and_then(Slice32::from_slice)
    }

    fn set_tapret_commitment(&mut self, commitment: impl Into<[u8; 32]>) -> Result<(), KeyError> {
        if self.has_tapret_commitment() {
            return Err(KeyError::OutputAlreadyHasCommitment);
        }

        self.proprietary.insert(
            ProprietaryKey::tapret_commitment(),
            commitment.into().to_vec(),
        );

        Ok(())
    }

    fn has_tapret_proof(&self) -> bool {
        self.proprietary
            .contains_key(&ProprietaryKey::tapret_proof())
    }

    fn tapret_proof(&self) -> Option<TaprootMerkleBranch> {
        let proof = self.proprietary.get(&ProprietaryKey::tapret_proof())?;
        TaprootMerkleBranch::from_slice(proof).ok()
    }
}
