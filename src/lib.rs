//! It can be useful to not have to worry about missing keys in a map.
//! If a key is requested that doesn't have a value a default value is simply returned.
//! This is exactly what this library provides.
//!

//! A clear use case of this is when counting the unique elements in a list.
//! Here you want to add one to the existing value in the map for that key.
//! This is a problem for the first adition when there's no value for the key yet.
//! With this library you can specify when creating the map that the default value should be zero.
//!

//! ```rust
//! # use defaultmap::*;
//! let nums = [1, 4, 3, 3, 4, 2, 4];
//! let mut counts:  DefaultHashMap<i32, i32> = DefaultHashMap::new(0);
//! // DefaultHashMap::default() is equivalent.
//!
//! for num in nums.into_iter() {
//!     counts[*num] += 1;
//! }
//!
//! println!("{:?}", counts);
//! // DefaultHashMap { map: {1: 1, 3: 2, 2: 1, 4: 3}, default: 0 }
//!
//! # assert_eq!(1, counts[1]);
//! # assert_eq!(1, counts[2]);
//! # assert_eq!(2, counts[3]);
//! # assert_eq!(3, counts[4]);
//! ```

#![recursion_limit="128"]
#[macro_use]

extern crate delegatemethod;

pub use hashmap::DefaultHashMap;


mod hashmap {
    use std::hash::Hash;
    use std::collections::HashMap;
    use std::collections::hash_map::*;
    use std::borrow::Borrow;
    use std::ops::{Index, IndexMut};

    #[derive(PartialEq, Eq, Debug, Default)]
    pub struct DefaultHashMap<K: Eq + Hash, V: Clone> {
        map: HashMap<K, V>,
        default: V,
    }

    impl<K: Eq + Hash, V: Clone> DefaultHashMap<K, V> {
        pub fn new(default: V) -> DefaultHashMap<K, V> {
            DefaultHashMap {
                map: HashMap::new(),
                default: default,
            }
        }

        pub fn get_mut(&mut self, key: K) -> &mut V {
            self.map.entry(key).or_insert(self.default.clone())
        }

        pub fn get<Q, QB: Borrow<Q>>(&self, key: QB) -> &V
            where K: Borrow<Q>,
                  Q: ?Sized + Hash + Eq
        {
            self.map.get(key.borrow()).unwrap_or(&self.default)
        }
    }

    delegate_method!{
        impl<K: Eq + Hash, V: Clone> DefaultHashMap<K, V> {
            map as HashMap:
                pub fn capacity(&self) -> usize;
                pub fn reserve(&mut self, additional: usize);
                pub fn shrink_to_fit(&mut self);
                pub fn keys(&self) -> Keys<K, V>;
                pub fn values(&self) -> Values<K, V>;
                pub fn values_mut(&mut self) -> ValuesMut<K, V>;
                pub fn iter(&self) -> Iter<K, V>;
                pub fn iter_mut(&mut self) -> IterMut<K, V>;
                pub fn entry(&mut self, key: K) -> Entry<K, V>;
                pub fn len(&self) -> usize;
                pub fn is_empty(&self) -> bool;
                pub fn drain(&mut self) -> Drain<K, V>;
                pub fn clear(&mut self);
                pub fn insert(&mut self, k: K, v: V) -> Option<V>;
        }
    }

    macro_rules! q_func {
        ($name:ident, $K:ident, $($returns:ty),*) => (
            pub fn $name<Q>(&self, k: &Q) -> ($($returns),*)
                where K: Borrow<Q>,
                      Q: Hash + Eq
            {

              self.map.$name(k)
            }
        )
    }

    macro_rules! q_func_mut {
        ($name:ident, $K:ident, $($returns:ty),*) => (
            pub fn $name<Q>(&mut self, k: &Q) -> ($($returns),*)
                where K: Borrow<Q>,
                      Q: Hash + Eq
            {

              self.map.$name(k)
            }
        )
    }


    impl<K: Eq + Hash, V: Clone> DefaultHashMap<K, V> {
        q_func!(contains_key, K, bool);
        q_func_mut!(remove, K, Option<V>);
    }


    impl<'a, K: Eq + Hash, KB: Borrow<K>, V: Clone> Index<KB> for DefaultHashMap<K, V> {
        type Output = V;

        fn index(&self, index: KB) -> &V {
            self.get(index)
        }
    }

    impl<K: Eq + Hash, V: Clone> IndexMut<K> for DefaultHashMap<K, V> {
        #[inline]
        fn index_mut(&mut self, index: K) -> &mut V {
            self.get_mut(index)
        }
    }


}


#[cfg(test)]
mod tests {
    use super::DefaultHashMap;

    #[test]
    fn add() {
        let mut map: DefaultHashMap<i32, i32> = DefaultHashMap::default();
        *map.get_mut(0) += 1;
        map[1] += 4;
        map[2] = map[0] + map.get(&1);
        assert_eq!(*map.get(0), 1);
        assert_eq!(*map.get(&0), 1);
        assert_eq!(map[0], 1);
        assert_eq!(map[&0], 1);
        assert_eq!(*map.get(&1), 4);
        assert_eq!(*map.get(&2), 5);
        assert_eq!(*map.get(999), 0);
        assert_eq!(*map.get(&999), 0);
        assert_eq!(map[999], 0);
        assert_eq!(map[&999], 0);
    }

    #[test]
    fn counter() {
        let nums = [1, 4, 3, 3, 4, 2, 4];
        let mut counts: DefaultHashMap<i32, i32> = DefaultHashMap::default();
        for num in nums.into_iter() {
            counts[*num] += 1;
        }

        assert_eq!(1, counts[1]);
        assert_eq!(1, counts[2]);
        assert_eq!(2, counts[3]);
        assert_eq!(3, counts[4]);
    }
}
