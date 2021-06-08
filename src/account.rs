use sp_core::{Pair, Public};
use sp_runtime::traits::{IdentifyAccount, Verify};
use sugarfunge_runtime::{AccountId, Signature};

/// Generate a crypto public from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Generate a crypto pair from seed.
pub fn get_pair_from_seed<TPublic: Pair>(seed: &str) -> TPublic::Pair {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}
