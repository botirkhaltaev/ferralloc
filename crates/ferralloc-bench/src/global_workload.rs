use std::{collections::HashMap, hint::black_box, sync::Arc};

pub fn vec_push_clear(rounds: usize, len: usize) -> usize {
    let mut checksum = 0_usize;
    for round in 0..rounds {
        let mut values = Vec::with_capacity(len);
        for i in 0..len {
            values.push(i ^ round);
        }
        checksum ^= values.iter().copied().sum::<usize>();
        values.clear();
        black_box(values);
    }
    black_box(checksum)
}

pub fn vec_many_small(rounds: usize, count: usize) -> usize {
    let mut checksum = 0_usize;
    for round in 0..rounds {
        let mut values = Vec::with_capacity(count);
        for i in 0..count {
            values.push(vec![((i ^ round) & 0xff) as u8; (i % 128) + 1]);
        }
        checksum ^= values.iter().map(Vec::len).sum::<usize>();
        black_box(values);
    }
    black_box(checksum)
}

pub fn string_building(rounds: usize, count: usize) -> usize {
    let mut checksum = 0_usize;
    for round in 0..rounds {
        let mut text = String::new();
        for i in 0..count {
            text.push_str("item");
            text.push_str(&(i ^ round).to_string());
            text.push(';');
        }
        checksum ^= text.len();
        black_box(text);
    }
    black_box(checksum)
}

pub fn hashmap_insert_remove(rounds: usize, count: usize) -> usize {
    let mut checksum = 0_usize;
    for round in 0..rounds {
        let mut map = HashMap::with_capacity(count);
        for i in 0..count {
            map.insert(i, i ^ round);
        }
        for i in (0..count).step_by(2) {
            checksum ^= map.remove(&i).unwrap_or_default();
        }
        checksum ^= map.len();
        black_box(map);
    }
    black_box(checksum)
}

pub fn arc_clone_drop(rounds: usize, count: usize) -> usize {
    let mut checksum = 0_usize;
    for round in 0..rounds {
        let value = Arc::new(vec![round as u8; 1024]);
        let mut clones = Vec::with_capacity(count);
        for _ in 0..count {
            clones.push(Arc::clone(&value));
        }
        checksum ^= Arc::strong_count(&value);
        black_box(clones);
    }
    black_box(checksum)
}

pub fn mixed_collections(rounds: usize, count: usize) -> usize {
    vec_push_clear(rounds, count)
        ^ vec_many_small(rounds, count / 4)
        ^ string_building(rounds, count / 2)
        ^ hashmap_insert_remove(rounds, count / 2)
        ^ arc_clone_drop(rounds, count / 2)
}
