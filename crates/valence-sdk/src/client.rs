use crate::{Result, SdkError};
use anchor_client::{Client, Cluster, Program};
use anchor_lang::prelude::*;
use solana_sdk::{
    commitment_config::CommitmentConfig,
    signature::Keypair,
    signer::Signer,
};
use std::rc::Rc;

/// Valence client for interacting with the protocol
pub struct ValenceClient {
    pub client: Client<Rc<Keypair>>,
    pub payer: Rc<Keypair>,
    pub valence_kernel: Program<Rc<Keypair>>,
}

impl ValenceClient {
    /// Create a new Valence client
    pub fn new(
        cluster: Cluster,
        payer: Rc<Keypair>,
        commitment: Option<CommitmentConfig>,
    ) -> Result<Self> {
        let client = Client::new_with_options(
            cluster,
            payer.clone(),
            commitment.unwrap_or(CommitmentConfig::confirmed()),
        );

        let valence_kernel = client.program(valence_kernel::ID)?;

        Ok(Self {
            client,
            payer,
            valence_kernel,
        })
    }

    /// Get the payer's public key
    pub fn payer(&self) -> Pubkey {
        self.payer.pubkey()
    }

    /// Get the valence kernel program
    pub fn program(&self) -> &Program<Rc<Keypair>> {
        &self.valence_kernel
    }

    /// Get an account
    pub fn get_account<T: AccountDeserialize>(&self, address: &Pubkey) -> Result<T> {
        self.valence_kernel
            .account(*address)
            .map_err(|_| SdkError::AccountNotFound(address.to_string()))
    }
}