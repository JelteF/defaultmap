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
//! // DefaultHashMap::new() is equivalent.
//!
//! for num in nums.into_iter() {
//!     counts[num] += 1;
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
//! // DefaultHashMap::new() is equivalent.
//!
//! for (l, r) in synonym_tuples.into_iter() {
//!     synonym_map[l].push(r);
//!     synonym_map[r].push(l);
//! }
//!
//! assert_eq!(synonym_map["good"], vec!["nice"]);
//! assert_eq!(synonym_map["nice"], vec!["sweet", "entertaining", "good"]);
//! assert_eq!(synonym_map["evil"], Vec::<&str>::new());
//! ```

#![cfg_attr(docsrs, feature(doc_auto_cfg))]
#![cfg_attr(any(not(docsrs), ci), deny(rustdoc::all))]

mod default_fn;

pub use default_fn::DefaultFn;

mod btreemap;
mod hashmap;

pub use btreemap::DefaultBTreeMap;
pub use hashmap::DefaultHashMap;
