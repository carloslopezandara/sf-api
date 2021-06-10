use codec::Decode;
use sp_core::H256 as Hash;
use sp_core::{sr25519};
use std::sync::mpsc::channel;
use substrate_api_client::{utils::FromHexString, Api};
use sugarfunge_runtime::Event;
use system::EventRecord;

pub fn wait_for_event<F, R>(api: Api<sr25519::Pair>, handler: F) -> Option<R>
where
    F: Fn(&Event) -> Option<R>,
{
    let (events_in, events_out) = channel();
    api.subscribe_events(events_in).unwrap();

    let start_time = std::time::Instant::now();

    while start_time.elapsed().as_secs() < 20 {
        let event_str = events_out.recv().unwrap();
        let unhex = Vec::from_hex(event_str).unwrap();
        let mut er_enc = unhex.as_slice();
        let event_records = Vec::<EventRecord<Event, Hash>>::decode(&mut er_enc);
        if let Ok(event_records) = event_records {
            for event_record in &event_records {
                if let Some(handled) = handler(&event_record.event) {
                    return Some(handled);
                }
            }
        }
    }

    None
}
