use bonsai_platform::portable::collect_process_tree;
use std::io::{BufRead, BufReader};
use std::process::{Command, Stdio};
use std::thread;
use std::time::Duration;

#[test]
fn live_parent_and_child_are_both_in_the_scoped_process_total() {
    let mut parent = Command::new(env!("CARGO_BIN_EXE_bonsai-accounting-fixture"))
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn live accounting fixture");
    let stdout = parent.stdout.take().expect("fixture stdout");
    let mut line = String::new();
    BufReader::new(stdout)
        .read_line(&mut line)
        .expect("read process identities");
    let identities = line
        .split_whitespace()
        .map(str::parse::<u32>)
        .collect::<Result<Vec<_>, _>>()
        .expect("numeric process identities");
    assert_eq!(identities.len(), 2);
    thread::sleep(Duration::from_millis(100));

    let snapshot = collect_process_tree(identities[0]).expect("live process-tree snapshot");
    assert_eq!(snapshot.root_process_id, identities[0]);
    assert!(snapshot.observed_process_ids.contains(&identities[0]));
    assert!(snapshot.observed_process_ids.contains(&identities[1]));
    assert!(snapshot.process_count >= 2);
    assert!(snapshot.resident_memory_bytes > 0);
    parent.wait().expect("wait parent fixture");
}
