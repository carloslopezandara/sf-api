use crate::account::*;
use crate::command::*;
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
use system::EventRecord;

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
    let owner_account_id = get_account_id_from_seed::<sr25519::Public>(&req.seed);
    let api = Api::new(node)
        .map(|api| api.set_signer(owner.clone()))
        .unwrap();
    let (events_in, events_out) = channel();
    api.subscribe_events(events_in).unwrap();

    // Information for Era for mortal transactions
    let head = api.get_finalized_head().unwrap().unwrap();
    let h: Header = api.get_header(Some(head)).unwrap().unwrap();
    let period = 5;

    println!("Owner: {} create collection: {}", owner.public(), req.name);

    let xt: UncheckedExtrinsicV4<_> = compose_extrinsic_offline!(
        api.clone().signer.unwrap(),
        Call::NFT(NFTCall::create_collection(req.name.as_bytes().into())),
        api.get_nonce().unwrap(),
        Era::mortal(period, h.number.into()),
        api.genesis_hash,
        head,
        api.runtime_version.spec_version,
        api.runtime_version.transaction_version
    );
    println!("[+] Composed Extrinsic:\n {:?}\n", xt);

    let start_time = std::time::Instant::now();

    // Send and watch extrinsic until in block
    let collection_id: Option<u64> = web::block::<_, _, ()>(move || {
        let blockh = api
            .send_extrinsic(xt.hex_encode(), XtStatus::InBlock)
            .unwrap();
        println!("[+] Transaction got included in block {:?}", blockh);

        while start_time.elapsed().as_secs() < 20 {
            let event_str = events_out.recv().unwrap();
            let unhex = Vec::from_hex(event_str).unwrap();
            let mut er_enc = unhex.as_slice();
            let event_records = Vec::<EventRecord<Event, Hash>>::decode(&mut er_enc);
            if let Ok(event_records) = event_records {
                for event_record in &event_records {
                    match &event_record.event {
                        Event::sugarfunge_nft(nft_event) => match &nft_event {
                            sugarfunge_nft::Event::CollectionCreated(
                                new_collection_id,
                                account_id,
                            ) if *account_id == owner_account_id => {
                                println!("[+] Event: {:?}", nft_event);
                                return Ok(Some(*new_collection_id));
                            }
                            _ => (),
                        },
                        _ => (),
                    }
                }
            }
        }

        Ok(None)

        // Some(collection_id)
    })
    .await
    .unwrap();

    Ok(HttpResponse::Ok().json(CreateCollectionOutput {
        collection_id: collection_id.unwrap(),
    }))
}
