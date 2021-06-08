// use hex_literal::hex;
// use sc_service::ChainType;
// use sc_telemetry::TelemetryEndpoints;
use serde_json::json;
// use sp_consensus_aura::sr25519::AuthorityId as AuraId;
use sp_core::crypto::UncheckedInto;
use sp_core::{sr25519, Pair, Public};
// use sp_finality_grandpa::AuthorityId as GrandpaId;
use sp_runtime::traits::{IdentifyAccount, Verify};
use sugarfunge_runtime::{
    AccountId, AuraConfig, Balance, BalancesConfig, CurrencyId, CurrencyTokenConfig, GenesisConfig,
    GrandpaConfig, OrmlTokensConfig, Signature, SudoConfig, SystemConfig, TokenSymbol, DOLLARS,
    WASM_BINARY,
};

/// Generate a crypto pair from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}