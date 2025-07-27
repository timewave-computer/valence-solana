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
    pub valence_core: Program<Rc<Keypair>>,
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

        let valence_core = client.program(valence_core::ID)?;

        Ok(Self {
            client,
            payer,
            valence_core,
        })
    }

    /// Get the payer's public key
    pub fn payer(&self) -> Pubkey {
        self.payer.pubkey()
    }

    /// Get the valence core program
    pub fn program(&self) -> &Program<Rc<Keypair>> {
        &self.valence_core
    }

    /// Get an account
    pub fn get_account<T: AccountDeserialize>(&self, address: &Pubkey) -> Result<T> {
        self.valence_core
            .account(*address)
            .map_err(|_| SdkError::AccountNotFound(address.to_string()))
    }
}