use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Result};
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use sp_core::{crypto::Pair, sr25519};
use sp_keyring::AccountKeyring;
use sp_runtime::MultiAddress;
use structopt::StructOpt;
use substrate_api_client::{compose_extrinsic_offline, Api, UncheckedExtrinsicV4, XtStatus};
use sugarfunge_runtime::{BalancesCall, Call, Header};
use url::Url;

#[derive(StructOpt, Debug)]
#[structopt(name = "sf-api")]
struct Opt {
    #[structopt(
        short = "s",
        long = "node-server",
        default_value = "ws://127.0.0.1:9944"
    )]
    node_server: Url,
    #[structopt(short = "l", long = "listen", default_value = "http://127.0.0.1:4000")]
    listen: Url,
}

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
struct FundAccountOutput {
    amount: u128,
}

#[derive(Serialize, Deserialize)]
struct AccountBalanceInput {
    seed: String,
}

#[derive(Serialize, Deserialize)]
struct AccountBalanceOutput {
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
    let node: String = get_node_url_from_opt();
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

    Ok(HttpResponse::Ok().json(FundAccountOutput { amount }))
}

async fn account_balance(req: web::Json<AccountBalanceInput>) -> Result<HttpResponse> {
    let node: String = get_node_url_from_opt();
    let from = account::get_pair_from_seed::<sr25519::Pair>(&req.seed);
    let who = account::get_account_id_from_seed::<sr25519::Public>(&req.seed);

    let api = Api::new(node).map(|api| api.set_signer(from)).unwrap();
    let mut amount = 0;
    if let Ok(Some(account_data)) = api.get_account_data(&who) {
        amount = account_data.free;
    }

    println!("AccountId: {}  Balance: {}", who, amount);

    Ok(HttpResponse::Ok().json(AccountBalanceOutput { amount }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();

    HttpServer::new(|| {
        App::new()
            .route("/pair", web::post().to(create_pair))
            .route("/fund", web::post().to(fund_account))
            .route("/balance", web::get().to(account_balance))
    })
    .bind((opt.listen.host_str().unwrap(), opt.listen.port().unwrap()))?
    .run()
    .await
}

pub fn get_node_url_from_opt() -> String {
    let opt = Opt::from_args();
    opt.node_server.into()
}
