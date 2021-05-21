use clap::{load_yaml, App};
use sp_core::crypto::Pair;
use sp_keyring::AccountKeyring;
use sugarfunge_runtime::{BalancesCall, Call, Header};

use substrate_api_client::{compose_extrinsic, Api, UncheckedExtrinsicV4, XtStatus};

fn main() {
    env_logger::init();
    // let url: String = "wss://node.virse.io".into();
    let url: String = "ws://127.0.0.1:9944".into();

    // initialize api and set the signer (sender) that is used to sign the extrinsics
    let from = AccountKeyring::Alice.pair();
    let api = Api::new(url).map(|api| api.set_signer(from)).unwrap();

    // set the recipient
    let to = AccountKeyring::Eve.to_account_id();

    // call Balances::transfer
    // the names are given as strings
    #[allow(clippy::redundant_clone)]
    let xt: UncheckedExtrinsicV4<_> = compose_extrinsic!(
        api.clone(),
        "Balances",
        "transfer",
        GenericAddress::Id(to),
        Compact(4369000000000000000000 as u128)
    );

    println!("[+] Composed Extrinsic:\n {:?}\n", xt);

    // send and watch extrinsic until InBlock
    let tx_hash = api
        .send_extrinsic(xt.hex_encode(), XtStatus::InBlock)
        .unwrap();
    println!("[+] Transaction got included. Hash: {:?}", tx_hash);
}
