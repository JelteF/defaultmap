use derive_more::Debug;
use std::borrow::Borrow;
use std::collections::hash_map::*;
use std::collections::HashMap;
use std::collections::TryReserveError;
use std::hash::Hash;
use std::iter::{FromIterator, IntoIterator};
use std::ops::{Index, IndexMut};

use crate::DefaultFn;

/// A `HashMap` that returns a default when keys are accessed that are not present.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefaultHashMap<K: Eq + Hash, V> {
    map: HashMap<K, V>,
    default: V,
    #[debug(skip)]
    #[cfg_attr(feature = "with-serde", serde(skip))]
    default_fn: Box<dyn DefaultFn<V>>,
}

impl<K: Eq + Hash, V: PartialEq> PartialEq for DefaultHashMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.map == other.map && self.default == other.default
    }
}

impl<K: Eq + Hash, V: Eq> Eq for DefaultHashMap<K, V> {}

impl<K: Eq + Hash, V: Default> DefaultHashMap<K, V> {
    /// The `new()` constructor creates an empty DefaultHashMap with the default of `V`
    /// as the default for missing keys.
    /// This is desired default for most use cases, if your case requires a
    /// different default you should use the `with_default()` constructor.
    pub fn new() -> DefaultHashMap<K, V> {
        DefaultHashMap {
            map: HashMap::default(),
            default_fn: Box::new(|| V::default()),
            default: V::default(),
        }
    }
}

impl<K: Eq + Hash, V: Default> Default for DefaultHashMap<K, V> {
    /// The `default()` method is equivalent to `DefaultHashMap::new()`.
    fn default() -> DefaultHashMap<K, V> {
        DefaultHashMap::new()
    }
}

impl<K: Eq + Hash, V: Default> From<HashMap<K, V>> for DefaultHashMap<K, V> {
    /// If you already have a `HashMap` that you would like to convert to a
    /// `DefaultHashMap` you can use the `into()` method on the `HashMap` or the
    /// `from()` constructor of `DefaultHashMap`.
    /// The default value for missing keys will be `V::default()`,
    /// if this is not desired `DefaultHashMap::from_map_with_default()` should be used.
    fn from(map: HashMap<K, V>) -> DefaultHashMap<K, V> {
        DefaultHashMap {
            map,
            default_fn: Box::new(|| V::default()),
            default: V::default(),
        }
    }
}

impl<K: Eq + Hash, V> From<DefaultHashMap<K, V>> for HashMap<K, V> {
    /// The into method can be used to convert a `DefaultHashMap` back into a
    /// `HashMap`.
    fn from(default_map: DefaultHashMap<K, V>) -> HashMap<K, V> {
        default_map.map
    }
}

impl<K: Eq + Hash, V: Clone + 'static> DefaultHashMap<K, V> {
    /// Creates an empty `DefaultHashMap` with `default` as the default for missing keys.
    /// When the provided `default` is equivalent to `V::default()` it is preferred to use
    /// `DefaultHashMap::default()` instead.
    pub fn with_default(default: V) -> DefaultHashMap<K, V> {
        DefaultHashMap {
            map: HashMap::new(),
            default: default.clone(),
            default_fn: Box::new(move || default.clone()),
        }
    }

    /// Creates a `DefaultHashMap` based on a default and an already existing `HashMap`.
    /// If `V::default()` is the supplied default, usage of the `from()` constructor or the
    /// `into()` method on the original `HashMap` is preferred.
    pub fn from_map_with_default(map: HashMap<K, V>, default: V) -> DefaultHashMap<K, V> {
        DefaultHashMap {
            map,
            default: default.clone(),
            default_fn: Box::new(move || default.clone()),
        }
    }

    /// Changes the default value permanently or until `set_default()` is called again.
    pub fn set_default(&mut self, new_default: V) {
        self.default = new_default.clone();
        self.default_fn = Box::new(move || new_default.clone());
    }
}

impl<K: Eq + Hash, V> DefaultHashMap<K, V> {
    /// Returns a reference to the value stored for the provided key.
    /// If the key is not in the `DefaultHashMap` a reference to the default value is returned.
    /// Usually the `map[key]` method of retrieving keys is preferred over using `get` directly.
    /// This method accepts both references and owned values as the key.
    pub fn get<Q, QB: Borrow<Q>>(&self, key: QB) -> &V
    where
        K: Borrow<Q>,
        Q: ?Sized + Hash + Eq,
    {
        self.map.get(key.borrow()).unwrap_or(&self.default)
    }

    /// Returns the an owned version of the default value
    /// ```
    /// use defaultmap::DefaultHashMap;
    /// assert_eq!(DefaultHashMap::<String, i32>::new().get_default(), 0);
    /// ```
    pub fn get_default(&self) -> V {
        self.default_fn.call()
    }

    /// Creates an empty `DefaultHashMap` with `default_fn` as the default value generation
    /// function for missing keys. When the provided `default_fn` only calls clone on a value,
    /// using `DefaultHashMap::new` is preferred.
    pub fn with_fn(default_fn: impl DefaultFn<V> + 'static) -> DefaultHashMap<K, V> {
        DefaultHashMap {
            map: HashMap::new(),
            default: default_fn.call(),
            default_fn: Box::new(default_fn),
        }
    }

    /// Creates a `DefaultHashMap` based on an existing map and using `default_fn` as the default
    /// value generation function for missing keys. When the provided `default_fn` is equivalent to
    /// V::default(), then using `DefaultHashMap::from(map)` is preferred.
    pub fn from_map_with_fn(
        map: HashMap<K, V>,
        default_fn: impl DefaultFn<V> + 'static,
    ) -> DefaultHashMap<K, V> {
        DefaultHashMap {
            map,
            default: default_fn.call(),
            default_fn: Box::new(default_fn),
        }
    }
}

impl<K: Eq + Hash, V> DefaultHashMap<K, V> {
    /// Returns a mutable reference to the value stored for the provided key.
    /// If there is no value stored for the key the default value is first inserted for this
    /// key before returning the reference.
    /// Usually the `map[key] = new_val` is prefered over using `get_mut` directly.
    /// This method only accepts owned values as the key.
    pub fn get_mut(&mut self, key: K) -> &mut V {
        let entry = self.map.entry(key);
        match entry {
            Entry::Occupied(occupied) => occupied.into_mut(),
            Entry::Vacant(vacant) => vacant.insert(self.default_fn.call()),
        }
    }
}

/// Implements the `Index` trait so you can do `map[key]`.
/// Nonmutable indexing can be done both by passing a reference or an owned value as the key.
impl<K: Eq + Hash, KB: Borrow<K>, V> Index<KB> for DefaultHashMap<K, V> {
    type Output = V;

    fn index(&self, index: KB) -> &V {
        self.get(index)
    }
}

/// Implements the `IndexMut` trait so you can do `map[key] = val`.
/// Mutably indexing can only be done when passing an owned value as the key.
impl<K: Eq + Hash, V> IndexMut<K> for DefaultHashMap<K, V> {
    #[inline]
    fn index_mut(&mut self, index: K) -> &mut V {
        self.get_mut(index)
    }
}

// grcov-excl-start
/// These methods simply forward to the underlying `HashMap`, see that
/// [documentation](https://doc.rust-lang.org/std/collections/struct.HashMap.html) for
/// the usage of these methods.
impl<K: Eq + Hash, V> DefaultHashMap<K, V> {
    pub fn capacity(&self) -> usize {
        self.map.capacity()
    }
    #[inline]
    pub fn keys(&self) -> Keys<K, V> {
        self.map.keys()
    }
    #[inline]
    pub fn into_keys(self) -> IntoKeys<K, V> {
        self.map.into_keys()
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
    pub fn into_values(self) -> IntoValues<K, V> {
        self.map.into_values()
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
    pub fn retain<RF>(&mut self, f: RF)
    where
        RF: FnMut(&K, &mut V) -> bool,
    {
        self.map.retain(f)
    }
    #[inline]
    pub fn clear(&mut self) {
        self.map.clear()
    }
    #[inline]
    pub fn reserve(&mut self, additional: usize) {
        self.map.reserve(additional)
    }
    #[inline]
    pub fn try_reserve(&mut self, additional: usize) -> Result<(), TryReserveError> {
        self.map.try_reserve(additional)
    }
    #[inline]
    pub fn shrink_to_fit(&mut self) {
        self.map.shrink_to_fit()
    }
    #[inline]
    pub fn shrink_to(&mut self, min_capacity: usize) {
        self.map.shrink_to(min_capacity);
    }
    #[inline]
    pub fn entry(&mut self, key: K) -> Entry<K, V> {
        self.map.entry(key)
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
    pub fn remove_entry<Q: ?Sized>(&mut self, k: &Q) -> Option<(K, V)>
    where
        K: Borrow<Q>,
        Q: Hash + Eq,
    {
        self.map.remove_entry(k)
    }
}
// grcov-excl-stop

impl<K: Eq + Hash, V: Default> FromIterator<(K, V)> for DefaultHashMap<K, V> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        Self {
            map: HashMap::from_iter(iter),
            default: V::default(),
            default_fn: Box::new(|| V::default()),
        }
    }
}

/// The `defaulthashmap!` macro can be used to easily initialize a `DefaultHashMap` in the
/// following ways:
///
/// ```
/// # #[macro_use] extern crate defaultmap;
/// # use defaultmap::*;
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
/// ```
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
            $crate::DefaultHashMap::from_map_with_default(_map, $default)
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
        let normal_map = DefaultHashMap::<i32, i32>::with_default(5);
        assert_eq!(macro_map, normal_map);

        // filled hashmap with custom default
        let macro_map: DefaultHashMap<_, _> = defaulthashmap! {
            5,
            1 => 2,
            2 => 3,
        };
        let mut normal_map = DefaultHashMap::<_, _>::with_default(5);
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
    fn change_default() {
        let mut numbers: DefaultHashMap<i32, String> =
            DefaultHashMap::with_default("Mexico".to_string());

        assert_eq!("Mexico", numbers.get_mut(1));
        assert_eq!("Mexico", numbers.get_mut(2));
        assert_eq!("Mexico", numbers[3]);

        numbers.set_default("Cybernetics".to_string());
        assert_eq!("Mexico", numbers[1]);
        assert_eq!("Mexico", numbers[2]);
        assert_eq!("Cybernetics", numbers[3]);
        assert_eq!("Cybernetics", numbers[4]);
        assert_eq!("Cybernetics", numbers[5]);
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
        let _: DefaultHashMap<Hashable, Clonable> = DefaultHashMap::with_default(Clonable);
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

    #[test]
    fn with_fn() {
        let i: i32 = 20;
        let mut map = DefaultHashMap::with_fn(move || i);
        map[0] += 1;
        assert_eq!(21, map[0]);
        assert_eq!(20, map[1]);
    }

    #[test]
    fn from_map_with_fn() {
        let i: i32 = 20;
        let normal: HashMap<i32, i32> = vec![(0, 1), (2, 3)].into_iter().collect();
        let mut map = DefaultHashMap::from_map_with_fn(normal, move || i);
        map[0] += 1;
        assert_eq!(map[0], 2);
        assert_eq!(map[1], 20);
        assert_eq!(map[2], 3);
    }

    #[cfg(feature = "with-serde")]
    mod serde_tests {
        use super::*;

        #[test]
        fn deserialize_static() {
            let s = "{ 
                        \"map\" : 
                            {   \"foo\": 3, 
                                \"bar\": 5 
                            }, 
                        \"default\":15 
                    }";
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
            let h1: DefaultHashMap<&str, u64> = DefaultHashMap::with_default(42);
            let s = serde_json::to_string(&h1).unwrap();
            let h2: DefaultHashMap<&str, u64> = serde_json::from_str(&s).unwrap();
            assert_eq!(h2["answer"], 42);
        }

        #[test]
        fn std_hashmap() {
            let h1: DefaultHashMap<i32, i32> = defaulthashmap!(1=> 10, 2=> 20);
            let stdhm: std::collections::HashMap<i32, i32> = h1.clone().into();
            let s = serde_json::to_string(&stdhm).unwrap();
            let h2: DefaultHashMap<i32, i32> = DefaultHashMap::from_map_with_default(
                serde_json::from_str(&s).unwrap(),
                i32::default(),
            );
            assert_eq!(h1, h2);
        }
    }
}
