use std::collections::{hash_map::HashMap, vec_deque::VecDeque};

pub struct MruHashmap<K, V> {
    hash_map: HashMap<K, V>,
    deque: VecDeque<K>,
    capacity: usize,
}

impl<K, V> MruHashmap<K, V>
where
    K: std::cmp::Eq,
    K: std::hash::Hash,
    K: std::clone::Clone,
{
    pub fn new() -> Self {
        Self::with_capacity(100)
    }

    pub fn with_capacity(capacity: usize) -> Self {
        MruHashmap {
            hash_map: HashMap::with_capacity(capacity),
            deque: VecDeque::with_capacity(capacity),
            capacity,
        }
    }

    pub fn insert(&mut self, k: K, v: V) -> Option<V> {
        let contains = self.hash_map.contains_key(&k);
        if !contains && self.deque.len() == self.capacity {
            let front = self.deque.pop_front().unwrap();
            self.hash_map.remove(&front);
        }

        if !contains {
            self.deque.push_back(k.clone())
        }

        self.hash_map.insert(k, v)
    }

    pub fn get(&mut self, k: &K) -> Option<&V> {
        self.hash_map.get(&k)
    }
}

impl<K, V> Default for MruHashmap<K, V>
where
    K: std::cmp::Eq,
    K: std::hash::Hash,
    K: std::clone::Clone,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::MruHashmap;

    fn assert_len<K, V>(ch: &MruHashmap<K, V>, len: usize) {
        assert_eq!(ch.hash_map.len(), len);
        assert_eq!(ch.deque.len(), len);
    }

    fn assert_capacity<K, V>(ch: &MruHashmap<K, V>, len: usize) {
        assert!(ch.hash_map.capacity() >= len);
        assert!(ch.deque.capacity() >= len);
        assert_eq!(ch.capacity, len);
    }

    #[test]
    fn default_creates_an_empty_mru_hashmap() {
        let ch: MruHashmap<u32, u32> = Default::default();
        assert_len(&ch, 0);
        assert_capacity(&ch, 100)
    }

    #[test]
    fn new_creates_an_empty_mru_hashmap() {
        let ch: MruHashmap<u32, u32> = MruHashmap::new();
        assert_len(&ch, 0);
        assert_capacity(&ch, 100)
    }

    #[test]
    fn with_the_specific_capacity() {
        let ch: MruHashmap<u32, u32> = MruHashmap::with_capacity(101);
        assert_len(&ch, 0);
        assert_capacity(&ch, 101)
    }

    #[test]
    fn key_not_found_in_empty_mru_hashmap() {
        let mut ch: MruHashmap<u32, u32> = Default::default();
        assert_eq!(ch.get(&1), None);
    }

    #[test]
    fn insert_into_empty_mru_hashmap() {
        let mut ch: MruHashmap<u32, u32> = Default::default();
        assert_eq!(ch.insert(1, 2), None);
        assert_len(&ch, 1);
    }

    #[test]
    fn insert_new_item_into_non_empty_mru_hashmap() {
        let mut ch: MruHashmap<u32, u32> = Default::default();
        assert_eq!(ch.insert(1, 2), None);
        assert_eq!(ch.insert(3, 4), None);
        assert_len(&ch, 2);
    }

    #[test]
    fn insert_item_with_existing_key_and_same_value() {
        let mut ch: MruHashmap<u32, u32> = Default::default();
        assert_eq!(ch.insert(1, 2), None);
        assert_eq!(ch.insert(1, 2), Some(2));
        assert_len(&ch, 1);
    }

    #[test]
    fn insert_item_with_existing_key_and_updated_value() {
        let mut ch: MruHashmap<u32, u32> = Default::default();
        assert_eq!(ch.insert(1, 2), None);
        assert_eq!(ch.insert(1, 3), Some(2));
        assert_len(&ch, 1);
    }

    #[test]
    fn insert_new_item_when_at_capacity() {
        let mut ch: MruHashmap<u32, u32> = MruHashmap::with_capacity(2);
        ch.insert(1, 2);
        ch.insert(2, 3);
        ch.insert(3, 4);

        assert_len(&ch, 2);
        assert_eq!(ch.get(&1), None);
        assert_eq!(ch.get(&2), Some(&3));
        assert_eq!(ch.get(&3), Some(&4));
    }
}
