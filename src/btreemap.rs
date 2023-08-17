use derive_more::Debug;
use std::borrow::Borrow;
use std::collections::btree_map::*;
use std::collections::BTreeMap;
use std::iter::{FromIterator, IntoIterator};
use std::ops::{Index, IndexMut};

/// A `BTreeMap` that returns a default when keys are accessed that are not present.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "with-serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DefaultBTreeMap<K: Eq + Ord, V> {
    map: BTreeMap<K, V>,
    default: V,
    #[debug(skip)]
    #[cfg_attr(feature = "with-serde", serde(skip))]
    default_fn: Box<dyn crate::DefaultFn<V>>,
}

impl<K: Eq + Ord, V: PartialEq> PartialEq for DefaultBTreeMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.map == other.map && self.default == other.default
    }
}

impl<K: Eq + Ord, V: Eq> Eq for DefaultBTreeMap<K, V> {}

impl<K: Eq + Ord, V: Default + Clone> Default for DefaultBTreeMap<K, V> {
    /// The `default()` constructor creates an empty DefaultBTreeMap with the default of `V`
    /// as the default for missing keys.
    /// This is desired default for most use cases, if your case requires a
    /// different default you should use the `new()` constructor.
    fn default() -> DefaultBTreeMap<K, V> {
        DefaultBTreeMap {
            map: BTreeMap::default(),
            default_fn: Box::new(|| V::default()),
            default: V::default(),
        }
    }
}

impl<K: Eq + Ord, V: Default> From<BTreeMap<K, V>> for DefaultBTreeMap<K, V> {
    /// If you already have a `BTreeMap` that you would like to convert to a
    /// `DefaultBTreeMap` you can use the `into()` method on the `BTreeMap` or the
    /// `from()` constructor of `DefaultBTreeMap`.
    /// The default value for missing keys will be `V::default()`,
    /// if this is not desired `DefaultBTreeMap::new_with_map()` should be used.
    fn from(map: BTreeMap<K, V>) -> DefaultBTreeMap<K, V> {
        DefaultBTreeMap {
            map,
            default_fn: Box::new(|| V::default()),
            default: V::default(),
        }
    }
}

impl<K: Eq + Ord, V> From<DefaultBTreeMap<K, V>> for BTreeMap<K, V> {
    /// The into method can be used to convert a `DefaultBTreeMap` back into a
    /// `BTreeMap`.
    fn from(default_map: DefaultBTreeMap<K, V>) -> BTreeMap<K, V> {
        default_map.map
    }
}

impl<K: Eq + Ord, V: Clone + 'static> DefaultBTreeMap<K, V> {
    /// Creates an empty `DefaultBTreeMap` with `default` as the default for missing keys.
    /// When the provided `default` is equivalent to `V::default()` it is preferred to use
    /// `DefaultBTreeMap::default()` instead.
    pub fn new(default: V) -> DefaultBTreeMap<K, V> {
        DefaultBTreeMap {
            map: BTreeMap::new(),
            default: default.clone(),
            default_fn: Box::new(move || default.clone()),
        }
    }

    /// Creates a `DefaultBTreeMap` based on a default and an already existing `BTreeMap`.
    /// If `V::default()` is the supplied default, usage of the `from()` constructor or the
    /// `into()` method on the original `BTreeMap` is preferred.
    pub fn new_with_map(default: V, map: BTreeMap<K, V>) -> DefaultBTreeMap<K, V> {
        DefaultBTreeMap {
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

impl<K: Eq + Ord, V> DefaultBTreeMap<K, V> {
    /// Returns a reference to the value stored for the provided key.
    /// If the key is not in the `DefaultBTreeMap` a reference to the default value is returned.
    /// Usually the `map[key]` method of retrieving keys is preferred over using `get` directly.
    /// This method accepts both references and owned values as the key.
    pub fn get<Q, QB: Borrow<Q>>(&self, key: QB) -> &V
    where
        K: Borrow<Q>,
        Q: ?Sized + Ord + Eq,
    {
        self.map.get(key.borrow()).unwrap_or(&self.default)
    }

    /// Returns the an owned version of the default value
    pub fn get_default(&self) -> V {
        self.default_fn.call()
    }
}

impl<K: Eq + Ord, V> DefaultBTreeMap<K, V> {
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
impl<K: Eq + Ord, KB: Borrow<K>, V> Index<KB> for DefaultBTreeMap<K, V> {
    type Output = V;

    fn index(&self, index: KB) -> &V {
        self.get(index)
    }
}

/// Implements the `IndexMut` trait so you can do `map[key] = val`.
/// Mutably indexing can only be done when passing an owned value as the key.
impl<K: Eq + Ord, V> IndexMut<K> for DefaultBTreeMap<K, V> {
    #[inline]
    fn index_mut(&mut self, index: K) -> &mut V {
        self.get_mut(index)
    }
}

/// These methods simply forward to the underlying `BTreeMap`, see that
/// [documentation](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html) for
/// the usage of these methods.
impl<K: Eq + Ord, V> DefaultBTreeMap<K, V> {
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
        Q: ?Sized + Ord,
    {
        self.map.contains_key(k)
    }
    #[inline]
    pub fn remove<Q>(&mut self, k: &Q) -> Option<V>
    where
        K: Borrow<Q>,
        Q: ?Sized + Ord,
    {
        self.map.remove(k)
    }
    #[inline]
    pub fn retain<RF>(&mut self, f: RF)
    where
        RF: FnMut(&K, &mut V) -> bool,
    {
        self.map.retain(f)
    }
}

impl<K: Eq + Ord, V: Default> FromIterator<(K, V)> for DefaultBTreeMap<K, V> {
    fn from_iter<I>(iter: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
    {
        Self {
            map: BTreeMap::from_iter(iter),
            default: V::default(),
            default_fn: Box::new(|| V::default()),
        }
    }
}

/// The `defaultbtreemap!` macro can be used to easily initialize a `DefaultBTreeMap` in the
/// following ways:
///
/// ```
/// # #[macro_use] extern crate defaultmap;
/// # use defaultmap::*;
/// // An empty map with the default as default
/// let _: DefaultBTreeMap<i32, i32> = defaultbtreemap!{};
///
/// // An empty map with a specified default
/// let _: DefaultBTreeMap<i32, i32> = defaultbtreemap!{5};
///
/// // A prefilled map with the default as the default
/// let _: DefaultBTreeMap<i32, i32> = defaultbtreemap!{
///     1 => 10,
///     5 => 20,
///     6 => 30,
/// };
///
/// // A prefilled map with a custom default
/// let _: DefaultBTreeMap<i32, i32> = defaultbtreemap!{
///     5,
///     1 => 10,
///     5 => 20,
///     6 => 30,
/// };
/// ```
#[macro_export]
macro_rules! defaultbtreemap {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(defaultbtreemap!(@single $rest)),*]));
    // Copied almost verbatim from maplit
    (@btreemap $($key:expr => $value:expr),*) => {
        {
            let mut _map = ::std::collections::BTreeMap::new();
            $(
                _map.insert($key, $value);
            )*
            _map
        }
    };

    ($($key:expr => $value:expr,)+) => { defaultbtreemap!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _map = defaultbtreemap!(@btreemap $($key => $value),*);
            $crate::DefaultBTreeMap::from(_map)
        }
    };

    ($default:expr$(, $key:expr => $value:expr)+ ,) => { defaultbtreemap!($default, $($key => $value),+) };
    ($default:expr$(, $key:expr => $value:expr)*) => {
        {
            let _map = defaultbtreemap!(@btreemap $($key => $value),*);
            $crate::DefaultBTreeMap::new_with_map($default, _map)
        }
    };
}

#[cfg(test)]
mod tests {
    use super::DefaultBTreeMap;
    use std::collections::BTreeMap;

    #[test]
    fn macro_test() {
        // empty default
        let macro_map: DefaultBTreeMap<i32, i32> = defaultbtreemap! {};
        let normal_map = DefaultBTreeMap::<i32, i32>::default();
        assert_eq!(macro_map, normal_map);

        // with content
        let macro_map: DefaultBTreeMap<_, _> = defaultbtreemap! {
            1 => 2,
            2 => 3,
        };
        let mut normal_map = DefaultBTreeMap::<_, _>::default();
        normal_map[1] = 2;
        normal_map[2] = 3;
        assert_eq!(macro_map, normal_map);

        // empty with custom default
        let macro_map: DefaultBTreeMap<i32, i32> = defaultbtreemap! {5};
        let normal_map = DefaultBTreeMap::<i32, i32>::new(5);
        assert_eq!(macro_map, normal_map);

        // filled btreemap with custom default
        let macro_map: DefaultBTreeMap<_, _> = defaultbtreemap! {
            5,
            1 => 2,
            2 => 3,
        };
        let mut normal_map = DefaultBTreeMap::<_, _>::new(5);
        normal_map[1] = 2;
        normal_map[2] = 3;
        assert_eq!(macro_map, normal_map);
    }

    #[test]
    fn add() {
        let mut map: DefaultBTreeMap<i32, i32> = DefaultBTreeMap::default();
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
        let mut counts: DefaultBTreeMap<i32, i32> = DefaultBTreeMap::default();
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
        let mut numbers: DefaultBTreeMap<i32, String> = DefaultBTreeMap::new("Mexico".to_string());

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

        let mut synonym_map: DefaultBTreeMap<&str, Vec<&str>> = DefaultBTreeMap::default();

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

    #[derive(Ord, PartialOrd, Eq, PartialEq)]
    struct Orderable(i32);

    #[test]
    fn minimal_derives() {
        let _: DefaultBTreeMap<Orderable, Clonable> = DefaultBTreeMap::new(Clonable);
        let _: DefaultBTreeMap<Orderable, DefaultableValue> = DefaultBTreeMap::default();
    }

    #[test]
    fn from() {
        let normal: BTreeMap<i32, i32> = vec![(0, 1), (2, 3)].into_iter().collect();
        let mut default: DefaultBTreeMap<_, _> = normal.into();
        default.get_mut(4);
        assert_eq!(default[0], 1);
        assert_eq!(default[2], 3);
        assert_eq!(default[1], 0);
        assert_eq!(default[4], 0);
        let expected: BTreeMap<i32, i32> = vec![(0, 1), (2, 3), (4, 0)].into_iter().collect();
        assert_eq!(expected, default.into());
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
            let h: Result<DefaultBTreeMap<&str, i32>, _> = serde_json::from_str(&s);
            let h = h.unwrap();
            assert_eq!(h["foo"] * h["bar"], h["foobar"])
        }

        #[test]
        fn serialize_and_back() {
            let h1: DefaultBTreeMap<i32, u64> = defaultbtreemap!(1 => 10, 2 => 20, 3 => 30);
            let s = serde_json::to_string(&h1).unwrap();
            let h2: DefaultBTreeMap<i32, u64> = serde_json::from_str(&s).unwrap();
            assert_eq!(h2, h2);
            assert_eq!(h2[3], 30);
        }

        #[test]
        fn serialize_default() {
            let h1: DefaultBTreeMap<&str, u64> = DefaultBTreeMap::new(42);
            let s = serde_json::to_string(&h1).unwrap();
            let h2: DefaultBTreeMap<&str, u64> = serde_json::from_str(&s).unwrap();
            assert_eq!(h2["answer"], 42);
        }

        #[test]
        fn std_btreemap() {
            let h1: DefaultBTreeMap<i32, i32> = defaultbtreemap!(1=> 10, 2=> 20);
            let stdhm: std::collections::BTreeMap<i32, i32> = h1.clone().into();
            let s = serde_json::to_string(&stdhm).unwrap();
            let h2: DefaultBTreeMap<i32, i32> =
                DefaultBTreeMap::new_with_map(i32::default(), serde_json::from_str(&s).unwrap());
            assert_eq!(h1, h2);
        }
    }
}
