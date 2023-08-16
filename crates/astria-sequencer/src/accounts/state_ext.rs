use anyhow::{
    Context,
    Result,
};
use astria_proto::native::sequencer::v1alpha1::Address;
use async_trait::async_trait;
use borsh::{
    BorshDeserialize as _,
    BorshSerialize as _,
};
use hex::ToHex as _;
use penumbra_storage::{
    StateRead,
    StateWrite,
};
use tracing::{
    debug,
    instrument,
};

use crate::accounts::types::{
    Balance,
    Nonce,
};

const ACCOUNTS_PREFIX: &str = "accounts";

fn storage_key(address: &str) -> String {
    format!("{ACCOUNTS_PREFIX}/{address}")
}

pub(crate) fn balance_storage_key(address: Address) -> String {
    format!("{}/balance", storage_key(&address.encode_hex::<String>()))
}

pub(crate) fn nonce_storage_key(address: Address) -> String {
    format!("{}/nonce", storage_key(&address.encode_hex::<String>()))
}

#[async_trait]
pub(crate) trait StateReadExt: StateRead {
    #[instrument(skip(self))]
    async fn get_account_balance(&self, address: Address) -> Result<Balance> {
        let Some(bytes) = self
            .get_raw(&balance_storage_key(address))
            .await
            .context("failed reading raw account balance from state")?
        else {
            debug!("account balance not found, returning 0");
            return Ok(Balance::from(0));
        };
        let balance = Balance::try_from_slice(&bytes).context("invalid balance bytes")?;
        Ok(balance)
    }

    #[instrument(skip(self))]
    async fn get_account_nonce(&self, address: Address) -> Result<Nonce> {
        let bytes = self
            .get_raw(&nonce_storage_key(address))
            .await
            .context("failed reading raw account nonce from state")?;
        let Some(bytes) = bytes else {
            // the account has not yet been initialized; return 0
            return Ok(Nonce::from(0));
        };

        let nonce = Nonce::try_from_slice(&bytes).context("invalid nonce bytes")?;
        Ok(nonce)
    }
}

impl<T: StateRead> StateReadExt for T {}

#[async_trait]
pub(crate) trait StateWriteExt: StateWrite {
    #[instrument(skip(self))]
    fn put_account_balance(&mut self, address: Address, balance: Balance) -> Result<()> {
        let bytes = balance
            .try_to_vec()
            .context("failed to serialize balance")?;
        self.put_raw(balance_storage_key(address), bytes);
        Ok(())
    }

    #[instrument(skip(self))]
    fn put_account_nonce(&mut self, address: Address, nonce: Nonce) -> Result<()> {
        let bytes = nonce.try_to_vec().context("failed to serialize nonce")?;
        self.put_raw(nonce_storage_key(address), bytes);
        Ok(())
    }
}

impl<T: StateWrite> StateWriteExt for T {}
