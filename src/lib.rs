use std::collections::HashMap;

pub struct DefaultMap<K: Eq + std::hash::Hash, V: Clone> {
    map: HashMap<K, V>,
    default: V,
}

impl<K: Eq + std::hash::Hash, V: Clone> DefaultMap<K, V> {
    pub fn new(default: V) -> DefaultMap<K, V> {
        DefaultMap {
            map: HashMap::new(),
            default: default,
        }
    }

    pub fn get(&mut self, key: K) -> &mut V {
        self.map.entry(key).or_insert(self.default.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::DefaultMap;
    use std::collections::HashMap;

    #[test]
    fn add() {
        let mut map: DefaultMap<i32, i32> = DefaultMap::new(0);
        let mut normalmap: HashMap<i32, i32> = HashMap::new();
        *normalmap.entry(0).or_insert(0) += 1;
        *map.get(0) += 1;
        assert_eq!(*map.get(2), 0);
        assert_eq!(*map.get(0), 1);
    }
}
