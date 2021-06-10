use actix_web::{web, HttpResponse, Result};
use codec::Decode;
use serde::{Deserialize, Serialize};
use sp_core::H256 as Hash;
use sp_core::{crypto::Pair, sr25519};
use std::sync::mpsc::channel;
use substrate_api_client::{
    compose_extrinsic_offline, utils::FromHexString, Api, UncheckedExtrinsicV4, XtStatus,
};
use sugarfunge_runtime::{Call, Event, Header, NFTCall};
use crate::account::*;
use crate::command::*;

#[derive(Serialize, Deserialize)]
pub struct CreateCollectionInput {
    seed: String,
    name: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateCollectionOutput {
    collection_id: u64,
}

pub async fn create_collection(req: web::Json<CreateCollectionInput>) -> Result<HttpResponse> {
    let node: String = get_node_url_from_opt();
    let owner = get_pair_from_seed::<sr25519::Pair>(&req.seed);
    let api = Api::new(node)
        .map(|api| api.set_signer(owner.clone()))
        .unwrap();
    let (events_in, events_out) = channel();
    api.subscribe_events(events_in).unwrap();

    // Information for Era for mortal transactions
    let head = api.get_finalized_head().unwrap().unwrap();
    let h: Header = api.get_header(Some(head)).unwrap().unwrap();
    let period = 5;

    println!("Owner: {} creacte collection: {}", owner.public(), req.name);

    let xt: UncheckedExtrinsicV4<_> = compose_extrinsic_offline!(
        api.clone().signer.unwrap(),
        Call::NFT(NFTCall::create_collection(Vec::new())),
        api.get_nonce().unwrap(),
        Era::mortal(period, h.number.into()),
        api.genesis_hash,
        head,
        api.runtime_version.spec_version,
        api.runtime_version.transaction_version
    );
    println!("[+] Composed Extrinsic:\n {:?}\n", xt);

    // Send and watch extrinsic until in block
    let blockh = web::block::<_, _, ()>(move || {
        Ok(api
            .send_extrinsic(xt.hex_encode(), XtStatus::InBlock)
            .unwrap())
    })
    .await
    .unwrap();

    println!("[+] Transaction got included in block {:?}", blockh);

    let event_str = events_out.recv().unwrap();
    let _unhex = Vec::from_hex(event_str).unwrap();
    let mut _er_enc = _unhex.as_slice();
    let _events = Vec::<system::EventRecord<Event, Hash>>::decode(&mut _er_enc);
    println!("[+] Events {:?}", _events);

    Ok(HttpResponse::Ok().json(CreateCollectionOutput { collection_id: 0 }))
}