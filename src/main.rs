use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use serde_json::json;
#[cfg(feature = "std")]
#[cfg(feature = "full_crypto")]
use sp_core::{crypto::Pair, sr25519};
// use substrate_api_client::{compose_extrinsic, Api, UncheckedExtrinsicV4, XtStatus};
use sp_keyring::AccountKeyring;
use sp_runtime::MultiAddress;
use substrate_api_client::{compose_extrinsic_offline, Api, UncheckedExtrinsicV4, XtStatus};
use sugarfunge_runtime::{BalancesCall, Call, Header};
use substrate_api_client::extrinsic::codec::Compact;

use substrate_api_client::{compose_extrinsic};

mod account;
#[derive(Serialize, Deserialize)]
struct AccessToken {
    #[serde(rename = "accessToken")]
    access_token: String,
}

#[derive(Serialize, Deserialize)]
struct CreatePairResponse {
    seed: String,
    account: String,
}

#[derive(Serialize, Deserialize)]
struct FundAccountInput {
    seed: String,
}

#[derive(Serialize, Deserialize)]
struct FundAccountResponse {
    amount: u128,
}

async fn create_pair(_req: HttpRequest) -> Result<HttpResponse> {
    let seed = rand::thread_rng().gen::<[u8; 32]>();
    let seed = hex::encode(seed);
    let account = account::get_account_id_from_seed::<sr25519::Public>(&seed);
    Ok(HttpResponse::Ok().json(CreatePairResponse {
        seed,
        account: format!("{}", account),
    }))
}

async fn fund_account(req: web::Json<FundAccountInput>) -> Result<HttpResponse> {
    let node: String = "ws://127.0.0.1:9944".into();
    let from = AccountKeyring::Alice.pair();
    let api = Api::new(node).map(|api| api.set_signer(from)).unwrap();

    // Information for Era for mortal transactions
    let head = api.get_finalized_head().unwrap().unwrap();
    let h: Header = api.get_header(Some(head)).unwrap().unwrap();
    let period = 5;

    let to = account::get_account_id_from_seed::<sr25519::Public>(&req.seed);

    println!("AccountId To: {}", to);

    let amount = 123456789;

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

    // Send and watch extrinsic until in block
    let blockh = api
        .send_extrinsic(xt.hex_encode(), XtStatus::InBlock)
        .unwrap();
    println!("[+] Transaction got included in block {:?}", blockh);

    Ok(HttpResponse::Ok().json(FundAccountResponse { amount }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/pair", web::post().to(create_pair))
            .route("/fund", web::post().to(fund_account))
    })
    .bind(("127.0.0.1", 4000))?
    .run()
    .await
}

// {
//     "seed": "927e5d37f5a45951cc7576d274e337e72d09baedb674f83ee2d44cc67be32dba",
//     "account": "5GGpVCWoEzMjXh3tPEg3PAAjWyWJg7weQCh3YPxLdVRyG6RG"
// }
