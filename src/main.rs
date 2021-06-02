#[cfg(feature = "std")]
#[cfg(feature = "full_crypto")]
use sp_core::{crypto::Pair, sr25519};

use sugarfunge_runtime::{BalancesCall, Call, Header};

use substrate_api_client::{compose_extrinsic, Api, UncheckedExtrinsicV4, XtStatus};

use actix_web::{web, App, HttpRequest, HttpResponse, HttpServer, Responder, Result};

use rand::prelude::*;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
struct AccessToken {
    #[serde(rename = "accessToken")]
    access_token: String,
}

#[derive(Serialize, Deserialize)]
struct CreateAccountResponse {
    seed: String,
    public: String,
}

async fn create_account(_req: HttpRequest) -> Result<HttpResponse> {
    let seed = rand::thread_rng().gen::<[u8; 32]>();
    let pair = sp_core::sr25519::Pair::from_seed(&seed);
    Ok(HttpResponse::Ok().json(CreateAccountResponse {
        seed: hex::encode(seed),
        public: pair.public().to_string(),
    }))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .route("/account", web::post().to(create_account))
    })
    .bind(("127.0.0.1", 3000))?
    .run()
    .await
}
