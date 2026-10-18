//! Fuzz target for PermutationShard and ShardSet.
//! Tests: shard creation, next_target, send/recv.
//! Expected: no panics, no OOB, proper error handling.

#![no_main]
use libfuzzer_sys::fuzz_target;
use aegis_traversal::{PermutationShard, ShardSet};

fuzz_target!(|data: &[u8]| {
    if data.len() < 8 {
        return;
    }

    let shard_count = (u64::from_le_bytes(data[0..8].try_into().unwrap()) % 16) as usize + 1;

    // Test ShardSet creation
    if let Ok(mut set) = ShardSet::new_default(shard_count) {
        // Run bounded iterations
        for _ in 0..100 {
            let _ = set.send_all();
            let _ = set.recv_all();
        }

        // Test individual shard access
        for shard in set.shards_mut() {
            for _ in 0..50 {
                let _ = shard.next_target();
            }
        }
    }

    // Test single shard creation
    let id = (data[0] as usize) % 16;
    let _ = PermutationShard::new_default(id, 16);
});
