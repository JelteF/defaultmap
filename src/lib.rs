//! It can be useful to not have to worry about missing keys in a map.
//! If a key is requested that doesn't have a value a default value is simply returned.
//! This is exactly what this library provides.
//!
//! ## Examples

//! ### Counter
//! A clear use case of this is when counting the unique elements in a list.
//! Here you want to add one to the existing value in the map for that key.
//! This is a problem for the first addition when there's no value for the key yet.
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
//!

//! ### Synonym lists
//!
//! Another way the default map can be used is using a map filled with other collections, such as a
//! a `Vec`, a `HashMap` or even another default map.
//! Next follows some code to create a map where we start with tuples of synonyms and we end with a
//! map that contains the list of synonyms for each word.
//!
//! ```rust
//! # use defaultmap::*;
//!
//! let synonym_tuples = [
//!     ("nice", "sweet"),
//!     ("sweet", "candy"),
//!     ("nice", "entertaining"),
//!     ("nice", "good"),
//!     ("entertaining", "absorbing"),
//! ];
//!
//! let mut synonym_map: DefaultHashMap<&str, Vec<&str>> = DefaultHashMap::new(vec![]);
//! // DefaultHashMap::default() is equivalent.
//!
//! for &(l, r) in synonym_tuples.into_iter() {
//!     synonym_map[l].push(r);
//!     synonym_map[r].push(l);
//! }
//!
//! assert_eq!(synonym_map["good"], vec!["nice"]);
//! assert_eq!(synonym_map["nice"], vec!["sweet", "entertaining", "good"]);
//! assert_eq!(synonym_map["evil"], Vec::<&str>::new());
//! ```
//!

pub use hashmap::DefaultHashMap;


mod hashmap {
    use std::hash::Hash;
    use std::collections::HashMap;
    use std::collections::hash_map::*;
    use std::borrow::Borrow;
    use std::ops::{Index, IndexMut};

    /// A `HashMap` that has returns a default when keys are accessed that are not present.
    #[derive(PartialEq, Eq, Debug)]
    pub struct DefaultHashMap<K: Eq + Hash, V: Clone> {
        map: HashMap<K, V>,
        default: V,
    }

    impl<K: Eq + Hash, V: Default + Clone> Default for DefaultHashMap<K, V> {
        fn default() -> DefaultHashMap<K, V> {
            DefaultHashMap {
                map: HashMap::default(),
                default: V::default(),
            }
        }
    }

    impl<K: Eq + Hash, V: Clone> DefaultHashMap<K, V> {
        /// Creates an empty `DefaultHashmap` with `default` as the default for missing keys.
        /// When the provided `default` is equivalent to `V::default()` it is preferred to use
        /// `DefaultHashMap::default()` instead.
        pub fn new(default: V) -> DefaultHashMap<K, V> {
            DefaultHashMap {
                map: HashMap::new(),
                default: default,
            }
        }

        /// Creates a `DefaultHashMap` based on a default and an already existiting `HashMap`.
        pub fn new_with_map(default: V, map: HashMap<K, V>) -> DefaultHashMap<K, V> {
            DefaultHashMap {
                map: map,
                default: default,
            }
        }

        /// Returns a reference to the value stored for the provided key.
        /// If the key is not in the `DefaultHashMap` a reference to the default value is returned.
        /// Usually the `map[key]` method of retrieving keys is prefered over using `get` directly.
        /// This method accepts both references and owned values as the key.
        pub fn get<Q, QB: Borrow<Q>>(&self, key: QB) -> &V
            where K: Borrow<Q>,
                  Q: ?Sized + Hash + Eq
        {
            self.map.get(key.borrow()).unwrap_or(&self.default)
        }

        /// Returns a mutable reference to the value stored for the provided key.
        /// If there is no value stored for the key the default value is first inserted for this
        /// key before returning the reference.
        /// Usually the `map[key] = new_val` is prefered over using `get_mut` directly.
        /// This method only accepts owned values as the key.
        pub fn get_mut(&mut self, key: K) -> &mut V {
            self.map.entry(key).or_insert(self.default.clone())
        }
    }

    /// Implements the `Index` trait so you can do `map[key]`.
    /// Nonmutable indexing can be done both by passing a reference or an owned value as the key.
    impl<'a, K: Eq + Hash, KB: Borrow<K>, V: Clone> Index<KB> for DefaultHashMap<K, V> {
        type Output = V;

        fn index(&self, index: KB) -> &V {
            self.get(index)
        }
    }

    /// Implements the `IndexMut` trait so you can do `map[key] = val`.
    /// Mutably indexing can only be done when passing an owned value as the key.
    impl<K: Eq + Hash, V: Clone> IndexMut<K> for DefaultHashMap<K, V> {
        #[inline]
        fn index_mut(&mut self, index: K) -> &mut V {
            self.get_mut(index)
        }
    }



    /// These methods simply forward to the underlying `HashMap`, see that
    /// [documentation](https://doc.rust-lang.org/std/collections/struct.HashMap.html) for
    /// the usage of these methods.
    impl<K: Eq + Hash, V: Clone> DefaultHashMap<K, V> {
        pub fn capacity(&self) -> usize {
            self.map.capacity()
        }
        #[inline]
        pub fn reserve(&mut self, additional: usize) {
            self.map.reserve(additional)
        }
        #[inline]
        pub fn shrink_to_fit(&mut self) {
            self.map.shrink_to_fit()
        }
        #[inline]
        pub fn keys(&self) -> Keys<K, V> {
            self.map.keys()
        }
        #[inline]
        pub fn values(&self) -> Values<K, V> {
            self.map.values()
        }
        #[inline]
        pub fn values_mut(&mut self) -> ValuesMut<K, V> {
            self.map.values_mut()
        }
        #[inline]
        pub fn iter(&self) -> Iter<K, V> {
            self.map.iter()
        }
        #[inline]
        pub fn iter_mut(&mut self) -> IterMut<K, V> {
            self.map.iter_mut()
        }
        #[inline]
        pub fn entry(&mut self, key: K) -> Entry<K, V> {
            self.map.entry(key)
        }
        #[inline]
        pub fn len(&self) -> usize {
            self.map.len()
        }
        #[inline]
        pub fn is_empty(&self) -> bool {
            self.map.is_empty()
        }
        #[inline]
        pub fn drain(&mut self) -> Drain<K, V> {
            self.map.drain()
        }
        #[inline]
        pub fn clear(&mut self) {
            self.map.clear()
        }
        #[inline]
        pub fn insert(&mut self, k: K, v: V) -> Option<V> {
            self.map.insert(k, v)
        }
        #[inline]
        pub fn contains_key<Q>(&self, k: &Q) -> (bool)
            where K: Borrow<Q>,
                  Q: ?Sized + Hash + Eq
        {
            self.map.contains_key(k)
        }
        #[inline]
        pub fn remove<Q>(&mut self, k: &Q) -> (Option<V>)
            where K: Borrow<Q>,
                  Q: ?Sized + Hash + Eq
        {
            self.map.remove(k)
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
        assert_eq!(0, counts[5]);
    }

    #[test]
    fn synonyms() {
        let synonym_tuples = [
            ("nice", "sweet"),
            ("sweet", "candy"),
            ("nice", "entertaining"),
            ("nice", "good"),
            ("entertaining", "absorbing"),
        ];

        let mut synonym_map: DefaultHashMap<&str, Vec<&str>> = DefaultHashMap::default();

        for &(l, r) in synonym_tuples.into_iter() {
            synonym_map[l].push(r);
            synonym_map[r].push(l);
        }

        println!("{:#?}", synonym_map);
        assert_eq!(synonym_map["good"], vec!["nice"]);
        assert_eq!(synonym_map["nice"], vec!["sweet", "entertaining", "good"]);
        assert_eq!(synonym_map["evil"], Vec::<&str>::new());
        // assert!(false)
    }

    #[derive(Clone)]
    struct Clonable;

    #[derive(Default, Clone)]
    struct DefaultableValue;

    #[derive(Hash, Eq, PartialEq)]
    struct Hashable(i32);

    #[test]
    fn minimal_derives() {
        let minimal: DefaultHashMap<Hashable, Clonable> = DefaultHashMap::new(Clonable);
        let clonable: DefaultHashMap<Hashable, DefaultableValue> = DefaultHashMap::default();
    }

}
