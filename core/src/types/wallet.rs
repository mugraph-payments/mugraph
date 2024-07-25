use crate::{crypto::*, Hash};

use self::schnorr::Signature;

pub struct Note {
    /// The ID for the asset in the Cardano blockchain
    pub asset_id: Hash,
    /// The amount included in this note
    pub amount: u128,
    /// The secret for this note, used to prevent double-spend on the server
    pub nullifier: RistrettoPoint,
    /// Unblinded signature from the server from this note creation
    ///
    /// Equivalent to C in the protocol, returned by the server after minting or swapping
    /// assets.
    pub signature: Signature,
}
