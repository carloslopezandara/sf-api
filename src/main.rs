use account::*;
use actix_web::{web, App, HttpServer};
use command::*;
use nft::*;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

mod account;
mod command;
mod event;
mod nft;

#[derive(Serialize, Deserialize)]
struct AccessToken {
    #[serde(rename = "accessToken")]
    access_token: String,
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let opt = Opt::from_args();

    HttpServer::new(|| {
        App::new()
            .route("/pair", web::post().to(create_pair))
            .route("/fund", web::post().to(fund_account))
            .route("/balance", web::post().to(account_balance))
            .route("/collection", web::post().to(create_collection))
            .route("/test", web::post().to(test))
    })
    .bind((opt.listen.host_str().unwrap(), opt.listen.port().unwrap()))?
    .run()
    .await
}
