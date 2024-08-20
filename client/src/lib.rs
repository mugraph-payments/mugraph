pub mod prelude {
    pub use mugraph_core::{
        builder::{CoinSelectionStrategy, GreedyCoinSelection, TransactionBuilder},
        crypto,
        error::{Error, Result},
        types::{
            Hash, Keypair, Note, PublicKey, Request, Response, SecretKey, Signature, Transaction,
            V0Request, V0Response,
        },
    };
}
