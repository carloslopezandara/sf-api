use crate::account::*;
use crate::command::*;
use crate::event::wait_for_event;
use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use serde_json::{Value};
use sp_core::{crypto::Pair, sr25519};
use substrate_api_client::{compose_extrinsic_offline, Api, UncheckedExtrinsicV4, XtStatus};
use sugarfunge_runtime::{Call, Event, Header, NFTCall};

//This section allows to deconstruct the input received by the endpoint "/collection" and construct its output
#[derive(Serialize, Deserialize)]
pub struct CreateCollectionInput {
    input: CreateCollectionArg,
}

#[derive(Serialize, Deserialize)]
pub struct CreateCollectionArg {
    seed: String,
    name: String,
}

#[derive(Serialize, Deserialize)]
pub struct CreateCollectionOutput {
    collection_id: u64,
}

//Creates a collection associated with a seed and returns its ID
pub async fn create_collection(req: web::Json<CreateCollectionInput>) -> Result<HttpResponse> {
    let node: String = get_node_url_from_opt();
    let owner = get_pair_from_seed::<sr25519::Pair>(&req.input.seed);
    let owner_account_id = get_account_id_from_seed::<sr25519::Public>(&req.input.seed);
    let api = Api::new(node)
        .map(|api| api.set_signer(owner.clone()))
        .unwrap();

    // Information for Era for mortal transactions
    let head = api.get_finalized_head().unwrap().unwrap();
    let h: Header = api.get_header(Some(head)).unwrap().unwrap();
    let period = 5;

    println!(
        "Owner: {} create collection: {}",
        owner.public(),
        req.input.name
    );

    let xt: UncheckedExtrinsicV4<_> = compose_extrinsic_offline!(
        api.clone().signer.unwrap(),
        Call::NFT(NFTCall::create_collection(req.input.name.as_bytes().into())),
        api.get_nonce().unwrap(),
        Era::mortal(period, h.number.into()),
        api.genesis_hash,
        head,
        api.runtime_version.spec_version,
        api.runtime_version.transaction_version
    );
    println!("[+] Composed Extrinsic:\n {:?}\n", xt);

    // Send and watch extrinsic until in block
    let collection_id: Option<u64> = web::block::<_, _, ()>(move || {
        let blockh = api
            .send_extrinsic(xt.hex_encode(), XtStatus::InBlock)
            .unwrap();
        println!("[+] Transaction got included in block {:?}", blockh);

        Ok(wait_for_event(api, |event| match event {
            Event::sugarfunge_nft(nft_event) => match &nft_event {
                sugarfunge_nft::Event::CollectionCreated(new_collection_id, account_id)
                    if *account_id == owner_account_id =>
                {
                    println!("[+] Event: {:?}", nft_event);
                    return Some(*new_collection_id);
                }
                _ => None,
            },
            _ => None,
        }))
    })
    .await
    .unwrap();

    Ok(HttpResponse::Ok().json(CreateCollectionOutput {
        collection_id: collection_id.unwrap(),
    }))
}

//This section allows to deconstruct the input received by the endpoint "/mint" and construct its output

#[derive(Serialize, Deserialize)]
pub struct MintNftInput {
    input: MintNftArg,
}

#[derive(Serialize, Deserialize)]
pub struct MintNftArg {
    seed: String,
    collection_id: u64,
    metadata: Value, //Value means that metadata expects a json
    quantity: u32,
}

#[derive(Serialize, Deserialize)]
pub struct MintNftOutput {
    collection_id: u64,
    asset_ids: Vec<u64>,
}

//Mints new NFTs acording to the given quantity in a given collection and returns its IDs
pub async fn mint(req: web::Json<MintNftInput>) -> Result<HttpResponse> {
    let node: String = get_node_url_from_opt();
    let owner = get_pair_from_seed::<sr25519::Pair>(&req.input.seed);
    let owner_account_id = get_account_id_from_seed::<sr25519::Public>(&req.input.seed);
    let api = Api::new(node)
        .map(|api| api.set_signer(owner.clone()))
        .unwrap();

    // Information for Era for mortal transactions
    let head = api.get_finalized_head().unwrap().unwrap();
    let h: Header = api.get_header(Some(head)).unwrap().unwrap();
    let period = 5;

    println!(
        "Owner: {} mint NFT for collection: {}",
        owner.public(),
        req.input.collection_id
    );

    let metadata: Vec<u8> = serde_json::to_vec(&req.input.metadata).unwrap();

    let xt: UncheckedExtrinsicV4<_> = compose_extrinsic_offline!(
        api.clone().signer.unwrap(),
        Call::NFT(NFTCall::mint(
            req.input.collection_id,
            metadata.clone(),
            req.input.quantity
        )),
        api.get_nonce().unwrap(),
        Era::mortal(period, h.number.into()),
        api.genesis_hash,
        head,
        api.runtime_version.spec_version,
        api.runtime_version.transaction_version
    );
    println!("[+] Composed Extrinsic:\n {:?}\n", xt);

    // Send and watch extrinsic until in block
    let response: Option<(u64, Vec<u64>)> = web::block::<_, _, ()>(move || {
        let blockh = api
            .send_extrinsic(xt.hex_encode(), XtStatus::InBlock)
            .unwrap();
        println!("[+] Transaction got included in block {:?}", blockh);

        Ok(wait_for_event(api, |event| match event {
            Event::sugarfunge_nft(nft_event) => match &nft_event {
                sugarfunge_nft::Event::TokenMint(collection_id, asset_ids, account_id)
                    if *account_id == owner_account_id =>
                {
                    println!("[+] Event: {:?}", nft_event);
                    return Some((*collection_id, asset_ids.clone()));
                }
                _ => None,
            },
            _ => None,
        }))
    })
    .await
    .unwrap();

    let (collection_id, asset_ids) = response.unwrap();

    Ok(HttpResponse::Ok().json(MintNftOutput {
        collection_id: collection_id,
        asset_ids: asset_ids,
    }))
}
