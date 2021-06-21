use crate::command::*;
use actix_web::*;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use sp_core::{sr25519, Pair, Public};
use sp_keyring::AccountKeyring;
use sp_runtime::traits::{IdentifyAccount, Verify};
use sp_runtime::MultiAddress;
use substrate_api_client::{compose_extrinsic_offline, Api, UncheckedExtrinsicV4, XtStatus};
use sugarfunge_runtime::{AccountId, BalancesCall, Call, Header, Signature};
use std::str;

/// Generate a crypto public from seed.
pub fn get_from_seed<TPublic: Public>(seed: &str) -> <TPublic::Pair as Pair>::Public {
    TPublic::Pair::from_string(&format!("//{}", seed), None)
        .expect("static values are valid; qed")
        .public()
}

/// Generate a crypto pair from seed.
pub fn get_pair_from_seed<TPublic: Pair>(seed: &str) -> TPublic::Pair {
    TPublic::Pair::from_string(&format!("//{}", seed), None).expect("static values are valid; qed")
}

type AccountPublic = <Signature as Verify>::Signer;

/// Generate an account ID from seed.
pub fn get_account_id_from_seed<TPublic: Public>(seed: &str) -> AccountId
where
    AccountPublic: From<<TPublic::Pair as Pair>::Public>,
{
    AccountPublic::from(get_from_seed::<TPublic>(seed)).into_account()
}

#[derive(Serialize, Deserialize)]
pub struct CreatePairResponse {
    seed: String,
    account: String,
}

pub async fn create_pair(_req: HttpRequest) -> Result<HttpResponse> {
    let seed = rand::thread_rng().gen::<[u8; 32]>();
    let seed = hex::encode(seed);
    let account = get_account_id_from_seed::<sr25519::Public>(&seed);
    Ok(HttpResponse::Ok().json(CreatePairResponse {
        seed,
        account: format!("{}", account),
    }))
}

#[derive(Serialize, Deserialize)]
pub struct FundAccountInput {
    input: FundAccountArg,
}

#[derive(Serialize, Deserialize)]
pub struct FundAccountArg {
    seed: String,
}

#[derive(Serialize, Deserialize)]
pub struct FundAccountOutput {
    amount: u128,
}

pub async fn fund_account(req: web::Json<FundAccountInput>) -> Result<HttpResponse> {

    let node: String = get_node_url_from_opt();
    let from = AccountKeyring::Alice.pair();
    let api = Api::new(node).map(|api| api.set_signer(from)).unwrap();

    // Information for Era for mortal transactions
    let head = api.get_finalized_head().unwrap().unwrap();
    let h: Header = api.get_header(Some(head)).unwrap().unwrap();
    let period = 5;

    let to = get_account_id_from_seed::<sr25519::Public>(&req.input.seed);

    println!("AccountId To: {}", to);

    let amount = 10000000000000000000;

    let to = MultiAddress::Id(to);

    let xt: UncheckedExtrinsicV4<_> = compose_extrinsic_offline!(
        api.clone().signer.unwrap(),
        Call::Balances(BalancesCall::transfer(to.clone(), amount)),
        api.get_nonce().unwrap(),
        Era::mortal(period, h.number.into()),
        api.genesis_hash,
        head,
        api.runtime_version.spec_version,
        api.runtime_version.transaction_version
    );
    println!("[+] Composed Extrinsic:\n {:?}\n", xt);

    let blockh = web::block::<_, _, ()>(move || {
        Ok(api
            .send_extrinsic(xt.hex_encode(), XtStatus::InBlock)
            .unwrap())
    })
    .await
    .unwrap();
    println!("[+] Transaction got included in block {:?}", blockh);

    Ok(HttpResponse::Ok().json(FundAccountOutput { amount }))
}

#[derive(Serialize, Deserialize)]
pub struct AccountBalanceInput {
    input: AccountBalanceArg,
}

#[derive(Serialize, Deserialize)]
pub struct AccountBalanceArg {
    seed: String,
}

#[derive(Serialize, Deserialize)]
pub struct AccountBalanceOutput {
    amount: u128,
}

pub async fn account_balance(req: web::Json<AccountBalanceInput>) -> Result<HttpResponse> {
    let node: String = get_node_url_from_opt();
    let from = get_pair_from_seed::<sr25519::Pair>(&req.input.seed);
    let who = get_account_id_from_seed::<sr25519::Public>(&req.input.seed);
    let api = Api::new(node).map(|api| api.set_signer(from)).unwrap();

    let (amount, who) = web::block::<_, _, ()>(move || {
        let mut amount: u128 = 0;
        if let Ok(Some(account_data)) = api.get_account_data(&who) {
            amount = account_data.free;
        }
        Ok((amount, who))
    })
    .await
    .unwrap_or_default();

    println!("AccountId: {}  Balance: {}", who, amount);

    Ok(HttpResponse::Ok().json(AccountBalanceOutput { amount }))
}
