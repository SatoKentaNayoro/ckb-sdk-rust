//! The traits defined here is intent to describe the requirements of current
//!  library code and only implemented the trait in upper level code.

use ckb_types::{
    bytes::Bytes,
    core::TransactionView,
    packed::{CellOutput, Header, OutPoint, Transaction},
    H256,
};
use thiserror::Error;

/// Wallet errors
#[derive(Error, Debug)]
pub enum WalletError {
    #[error("the id is not found in the wallet")]
    IdNotFound,
    #[error("invalid message, reason: `{0}`")]
    InvalidMessage(String),
    #[error("get transaction dependency failed: `{0}`")]
    TxDep(#[from] TxDepProviderError),

    // maybe hardware wallet error or io error
    #[error("other error: `{0}`")]
    Other(#[from] Box<dyn std::error::Error>),
}

/// A wallet abstraction, support wallet type:
///    * secp256k1 ckb wallet
///    * secp256k1 eth wallet
///    * RSA wallet
///    * Hardware wallet
pub trait Wallet {
    /// typecial id are blake160(pubkey) and keccak256(pubkey)[12..20]
    fn match_id(&self, id: &[u8]) -> bool;

    /// `message` type is Bytes, because different algorithm have different length of message.
    ///   * secp256k1 => 256bits
    ///   * RSA       => 512bits (when key size is 1024bits)
    ///
    ///  For keystore case, `password` may read from prompt.
    ///  For ledger case, `password` will read from ledger device.
    fn sign(
        &self,
        id: &[u8],
        message: &[u8],
        tx: &TransactionView,
        // This is mainly for hardware wallet.
        tx_dep_provider: &mut dyn TransactionDependencyProvider,
    ) -> Result<Bytes, WalletError>;

    /// Verify a signature
    fn verify(&self, id: &[u8], message: &[u8], signature: Bytes) -> Result<bool, WalletError>;
}

/// Transaction dependency provider errors
#[derive(Error, Debug)]
pub enum TxDepProviderError {
    #[error("the resource is not found in the provider")]
    NotFound,
    #[error("other error: `{0}`")]
    Other(#[from] Box<dyn std::error::Error>),
}

/// Provider dependency information of a transaction:
///   * inputs
///   * cell_deps
///   * header_deps
pub trait TransactionDependencyProvider {
    // For verify certain cell belong to certain transaction
    fn get_tx(&mut self, tx_hash: H256) -> Result<Transaction, TxDepProviderError>;
    // For get the output information of inputs or cell_deps
    fn get_output(&mut self, out_point: OutPoint) -> Result<CellOutput, TxDepProviderError>;
    // For get the output data information of inputs or cell_deps
    fn get_output_data(&mut self, out_point: OutPoint) -> Result<Bytes, TxDepProviderError>;
    // For get the header information of header_deps
    fn get_header(&mut self, block_hash: H256) -> Result<Header, TxDepProviderError>;
}

/// An empty transaction dependency provider, this provider will return Err(NotFound) in all cases.
pub struct EmptyTxDepProvider;

impl TransactionDependencyProvider for EmptyTxDepProvider {
    fn get_tx(&mut self, _tx_hash: H256) -> Result<Transaction, TxDepProviderError> {
        Err(TxDepProviderError::NotFound)
    }
    fn get_output(&mut self, _out_point: OutPoint) -> Result<CellOutput, TxDepProviderError> {
        Err(TxDepProviderError::NotFound)
    }
    fn get_output_data(&mut self, _out_point: OutPoint) -> Result<Bytes, TxDepProviderError> {
        Err(TxDepProviderError::NotFound)
    }
    fn get_header(&mut self, _block_hash: H256) -> Result<Header, TxDepProviderError> {
        Err(TxDepProviderError::NotFound)
    }
}
