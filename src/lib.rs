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
//!
//! let nums = [1, 4, 3, 3, 4, 2, 4];
//! let mut counts: DefaultHashMap<i32, i32> = defaulthashmap!();
//! // DefaultHashMap::new(0) is equivalent.
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
//!
//! ```
//!

//! ### Synonym lists
//!
//! Another way the default map can be used is using a map filled with other collections, such as a
//! `Vec`, a `HashMap` or even another default map.
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
//! let mut synonym_map: DefaultHashMap<&str, Vec<&str>> = defaulthashmap!();
//! // DefaultHashMap::new(vec![]) is equivalent.
//!
//! for &(l, r) in synonym_tuples.into_iter() {
//!     synonym_map[l].push(r);
//!     synonym_map[r].push(l);
//! }
//!
//! assert_eq!(synonym_map["good"], vec!["nice"]);
//! assert_eq!(synonym_map["nice"], vec!["sweet", "entertaining", "good"]);
//! assert_eq!(synonym_map["evil"], Vec::<&str>::new());
//!
//! ```
//!

pub use hashmap::DefaultHashMap;

mod hashmap {
    use std::borrow::Borrow;
    use std::collections::hash_map::*;
    use std::collections::HashMap;
    use std::hash::Hash;
    use std::iter::{FromIterator, IntoIterator};
    use std::ops::{Index, IndexMut};

    /// A `HashMap` that returns a default when keys are accessed that are not present.
    #[derive(PartialEq, Eq, Clone, Debug)]
    #[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
    pub struct DefaultHashMap<K: Eq + Hash, V: Clone> {
        map: HashMap<K, V>,
        default: V,
    }

    impl<K: Eq + Hash, V: Default + Clone> Default for DefaultHashMap<K, V> {
        /// The `default()` constructor creates an empty DefaultHashMap with the default of `V`
        /// as the default for missing keys.
        /// This is desired default for most use cases, if your case requires a
        /// different default you should use the `new()` constructor.
        fn default() -> DefaultHashMap<K, V> {
            DefaultHashMap {
                map: HashMap::default(),
                default: V::default(),
            }
        }
    }

    impl<K: Eq + Hash, V: Default + Clone> From<HashMap<K, V>> for DefaultHashMap<K, V> {
        /// If you already have a `HashMap` that you would like to convert to a
        /// `DefaultHashMap` you can use the `into()` method on the `HashMap` or the
        /// `from()` constructor of `DefaultHashMap`.
        /// The default value for missing keys will be `V::default()`,
        /// if this is not desired `DefaultHashMap::new_with_map()` should be used.
        fn from(map: HashMap<K, V>) -> DefaultHashMap<K, V> {
            DefaultHashMap {
                map,
                default: V::default(),
            }
        }
    }

    impl<K: Eq + Hash, V: Clone> Into<HashMap<K, V>> for DefaultHashMap<K, V> {
        /// The into method can be used to convert a `DefaultHashMap` back into a
        /// `HashMap`.
        fn into(self) -> HashMap<K, V> {
            self.map
        }
    }

    impl<K: Eq + Hash, V: Clone> DefaultHashMap<K, V> {
        /// Creates an empty `DefaultHashmap` with `default` as the default for missing keys.
        /// When the provided `default` is equivalent to `V::default()` it is preferred to use
        /// `DefaultHashMap::default()` instead.
        pub fn new(default: V) -> DefaultHashMap<K, V> {
            DefaultHashMap {
                map: HashMap::new(),
                default,
            }
        }

        /// Creates a `DefaultHashMap` based on a default and an already existing `HashMap`.
        /// If `V::default()` is the supplied default, usage of the `from()` constructor or the
        /// `into()` method on the original `HashMap` is preferred.
        pub fn new_with_map(default: V, map: HashMap<K, V>) -> DefaultHashMap<K, V> {
            DefaultHashMap { map, default }
        }

        /// Returns a reference to the value stored for the provided key.
        /// If the key is not in the `DefaultHashMap` a reference to the default value is returned.
        /// Usually the `map[key]` method of retrieving keys is prefered over using `get` directly.
        /// This method accepts both references and owned values as the key.
        pub fn get<Q, QB: Borrow<Q>>(&self, key: QB) -> &V
        where
            K: Borrow<Q>,
            Q: ?Sized + Hash + Eq,
        {
            self.map.get(key.borrow()).unwrap_or(&self.default)
        }

        /// Returns a mutable reference to the value stored for the provided key.
        /// If there is no value stored for the key the default value is first inserted for this
        /// key before returning the reference.
        /// Usually the `map[key] = new_val` is prefered over using `get_mut` directly.
        /// This method only accepts owned values as the key.
        pub fn get_mut(&mut self, key: K) -> &mut V {
            let default = &self.default;
            self.map.entry(key).or_insert_with(|| default.clone())
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
        pub fn contains_key<Q>(&self, k: &Q) -> bool
        where
            K: Borrow<Q>,
            Q: ?Sized + Hash + Eq,
        {
            self.map.contains_key(k)
        }
        #[inline]
        pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
        where
            K: Borrow<Q>,
            Q: ?Sized + Hash + Eq,
        {
            self.map.remove(k)
        }
        #[inline]
        pub fn retain<F>(&mut self, f: F)
        where
            F: FnMut(&K, &mut V) -> bool,
        {
            self.map.retain(f)
        }
    }

    impl<K: Eq + Hash, V: Default + Clone> FromIterator<(K, V)> for DefaultHashMap<K, V> {
        fn from_iter<I>(iter: I) -> Self
        where
            I: IntoIterator<Item = (K, V)>,
        {
            Self {
                map: HashMap::from_iter(iter),
                default: V::default(),
            }
        }
    }
}

/// The `defaulthashmap!` macro can be used to easily initialize a `DefaultHashMap` in the
/// following ways:
///
/// ```
/// # #[macro_use] extern crate defaultmap;
/// # use defaultmap::*;
/// # fn main() {
/// // An empty map with the default as default
/// let _: DefaultHashMap<i32, i32> = defaulthashmap!{};
///
/// // An empty map with a specified default
/// let _: DefaultHashMap<i32, i32> = defaulthashmap!{5};
///
/// // A prefilled map with the default as the default
/// let _: DefaultHashMap<i32, i32> = defaulthashmap!{
///     1 => 10,
///     5 => 20,
///     6 => 30,
/// };
///
/// // A prefilled map with a custom default
/// let _: DefaultHashMap<i32, i32> = defaulthashmap!{
///     5,
///     1 => 10,
///     5 => 20,
///     6 => 30,
/// };
///
/// # }
#[macro_export]
macro_rules! defaulthashmap {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(defaulthashmap!(@single $rest)),*]));
    // Copied almost verbatim from maplit
    (@hashmap $($key:expr => $value:expr),*) => {
        {
            let _cap = defaulthashmap!(@count $($key),*);
            let mut _map = ::std::collections::HashMap::with_capacity(_cap);
            $(
                _map.insert($key, $value);
            )*
            _map
        }
    };

    ($($key:expr => $value:expr,)+) => { defaulthashmap!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _map = defaulthashmap!(@hashmap $($key => $value),*);
            $crate::DefaultHashMap::from(_map)
        }
    };

    ($default:expr$(, $key:expr => $value:expr)+ ,) => { defaulthashmap!($default, $($key => $value),+) };
    ($default:expr$(, $key:expr => $value:expr)*) => {
        {
            let _map = defaulthashmap!(@hashmap $($key => $value),*);
            $crate::DefaultHashMap::new_with_map($default, _map)
        }
    };
}

#[cfg(test)]
mod tests {
    use super::DefaultHashMap;
    use std::collections::HashMap;

    #[test]
    fn macro_test() {
        // empty default
        let macro_map: DefaultHashMap<i32, i32> = defaulthashmap! {};
        let normal_map = DefaultHashMap::<i32, i32>::default();
        assert_eq!(macro_map, normal_map);

        // with content
        let macro_map: DefaultHashMap<_, _> = defaulthashmap! {
            1 => 2,
            2 => 3,
        };
        let mut normal_map = DefaultHashMap::<_, _>::default();
        normal_map[1] = 2;
        normal_map[2] = 3;
        assert_eq!(macro_map, normal_map);

        // empty with custom default
        let macro_map: DefaultHashMap<i32, i32> = defaulthashmap! {5};
        let normal_map = DefaultHashMap::<i32, i32>::new(5);
        assert_eq!(macro_map, normal_map);

        // filled hashmap with custom default
        let macro_map: DefaultHashMap<_, _> = defaulthashmap! {
            5,
            1 => 2,
            2 => 3,
        };
        let mut normal_map = DefaultHashMap::<_, _>::new(5);
        normal_map[1] = 2;
        normal_map[2] = 3;
        assert_eq!(macro_map, normal_map);
    }

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
        for num in nums.iter() {
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

        for &(l, r) in synonym_tuples.iter() {
            synonym_map[l].push(r);
            synonym_map[r].push(l);
        }

        println!("{:#?}", synonym_map);
        assert_eq!(synonym_map["good"], vec!["nice"]);
        assert_eq!(synonym_map["nice"], vec!["sweet", "entertaining", "good"]);
        assert_eq!(synonym_map["evil"], Vec::<&str>::new());
    }

    #[derive(Clone)]
    struct Clonable;

    #[derive(Default, Clone)]
    struct DefaultableValue;

    #[derive(Hash, Eq, PartialEq)]
    struct Hashable(i32);

    #[test]
    fn minimal_derives() {
        let _: DefaultHashMap<Hashable, Clonable> = DefaultHashMap::new(Clonable);
        let _: DefaultHashMap<Hashable, DefaultableValue> = DefaultHashMap::default();
    }

    #[test]
    fn from() {
        let normal: HashMap<i32, i32> = vec![(0, 1), (2, 3)].into_iter().collect();
        let mut default: DefaultHashMap<_, _> = normal.into();
        default.get_mut(4);
        assert_eq!(default[0], 1);
        assert_eq!(default[2], 3);
        assert_eq!(default[1], 0);
        assert_eq!(default[4], 0);
        let expected: HashMap<i32, i32> = vec![(0, 1), (2, 3), (4, 0)].into_iter().collect();
        assert_eq!(expected, default.into());
    }

    #[cfg(feature = "with-serde")]
    mod serde_tests {
        use super::*;

        #[test]
        fn deserialize_static() {
            let s = "{ \"map\" : { \"foo\": 3, \"bar\": 5 }, \"default\":15 }";
            let h: Result<DefaultHashMap<&str, i32>, _> = serde_json::from_str(&s);
            let h = h.unwrap();
            assert_eq!(h["foo"] * h["bar"], h["foobar"])
        }

        #[test]
        fn serialize_and_back() {
            let h1: DefaultHashMap<i32, u64> = defaulthashmap!(1 => 10, 2 => 20, 3 => 30);
            let s = serde_json::to_string(&h1).unwrap();
            let h2: DefaultHashMap<i32, u64> = serde_json::from_str(&s).unwrap();
            assert_eq!(h2, h2);
            assert_eq!(h2[3], 30);
        }

        #[test]
        fn serialize_default() {
            let h1: DefaultHashMap<&str, u64> = DefaultHashMap::new(42);
            let s = serde_json::to_string(&h1).unwrap();
            let h2: DefaultHashMap<&str, u64> = serde_json::from_str(&s).unwrap();
            assert_eq!(h2["answer"], 42);
        }

        #[test]
        fn std_hashmap() {
            let h1: DefaultHashMap<i32, i32> = defaulthashmap!(1=> 10, 2=> 20);
            let stdhm: std::collections::HashMap<i32, i32> = h1.clone().into();
            let s = serde_json::to_string(&stdhm).unwrap();
            let h2: DefaultHashMap<i32, i32> =
                DefaultHashMap::new_with_map(i32::default(), serde_json::from_str(&s).unwrap());
            assert_eq!(h1, h2);
        }
    }
}
