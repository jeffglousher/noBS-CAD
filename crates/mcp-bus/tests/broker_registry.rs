//! Multi-window broker registry must be exercised through the bus (#12).

use std::thread;
use std::time::Duration;

use nbcad_mcp_bus::{
    list_via_bus, process_broker_one, register_via_bus, BrokerHandler, InMemoryBus, WindowEntry,
};

#[test]
fn register_and_list_windows_via_bus_only() {
    let bus = InMemoryBus::new();
    let worker_bus = bus.clone();
    let worker = thread::spawn(move || {
        let mut handler = BrokerHandler::new();
        // register A, register B, list
        for _ in 0..3 {
            process_broker_one(&worker_bus, &mut handler, Duration::from_secs(2))
                .expect("broker worker");
        }
        assert_eq!(handler.registry.list().len(), 2);
    });

    let reg_a = register_via_bus(
        &bus,
        &WindowEntry {
            document_id: "aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa".into(),
            window_id: Some("win-a".into()),
            title: Some("Part A".into()),
        },
        Duration::from_secs(2),
    )
    .unwrap();
    assert_eq!(reg_a["result"]["registered"], true);

    let reg_b = register_via_bus(
        &bus,
        &WindowEntry {
            document_id: "bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb".into(),
            window_id: Some("win-b".into()),
            title: Some("Part B".into()),
        },
        Duration::from_secs(2),
    )
    .unwrap();
    assert_eq!(reg_b["result"]["registered"], true);

    let listed = list_via_bus(&bus, Duration::from_secs(2)).unwrap();
    let windows = listed["result"]["windows"].as_array().unwrap();
    assert_eq!(windows.len(), 2);
    let ids: Vec<&str> = windows
        .iter()
        .filter_map(|window| window["document_id"].as_str())
        .collect();
    assert!(ids.contains(&"aaaaaaaa-aaaa-4aaa-8aaa-aaaaaaaaaaaa"));
    assert!(ids.contains(&"bbbbbbbb-bbbb-4bbb-8bbb-bbbbbbbbbbbb"));

    worker.join().unwrap();
}
