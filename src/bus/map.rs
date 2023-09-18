use std::cmp::Ordering;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::Debug;
use std::ops::RangeInclusive;

use crate::arch::Value;

type Range<Idx> = RangeInclusive<Idx>;

pub(super) trait Entry: Clone + Debug + Eq {}

impl<T> Entry for T where T: Clone + Debug + Eq {}

#[derive(Debug)]
pub(super) struct Bus<Idx, V>(BTreeMap<Idx, BTreeSet<Mapping<Idx, V>>>)
where
    Idx: Value,
    V: Entry;

impl<Idx, V> Bus<Idx, V>
where
    Idx: Value,
    V: Entry,
{
    #[allow(unused)]
    pub(super) fn new() -> Self {
        Self::default()
    }

    pub(super) fn clear(&mut self) {
        self.0.clear();
    }

    pub(super) fn map(&mut self, range: Range<Idx>, entry: V) {
        let map = Mapping::new(range, entry);
        self.0.entry(map.base()).or_default().insert(map.clone());
    }

    pub(super) fn get(&self, idx: Idx) -> Option<&Mapping<Idx, V>> {
        self.0
            .range(..=idx)
            .rev()
            .flat_map(|(_, maps)| maps.iter())
            .find(|map| map.contains(&idx))
    }

    #[allow(unused)]
    pub(super) fn iter(&self) -> impl Iterator + '_ {
        self.0.iter().flat_map(|(_, maps)| maps.iter())
    }
}

impl<Idx, V> Default for Bus<Idx, V>
where
    Idx: Value,
    V: Entry,
{
    fn default() -> Self {
        Self(BTreeMap::default())
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) struct Mapping<Idx, V>
where
    Idx: Value,
    V: Entry,
{
    pub(super) range: Range<Idx>,
    pub(super) entry: V,
}

impl<Idx, V> Mapping<Idx, V>
where
    Idx: Value,
    V: Entry,
{
    fn new(range: Range<Idx>, entry: V) -> Self {
        Self { range, entry }
    }

    pub(super) fn base(&self) -> Idx {
        *self.range.start()
    }

    fn len(&self) -> Idx {
        *self.range.end() - *self.range.start()
    }

    fn contains(&self, idx: &Idx) -> bool {
        self.range.contains(idx)
    }
}

impl<Idx, V> Ord for Mapping<Idx, V>
where
    Idx: Value,
    V: Entry,
{
    fn cmp(&self, other: &Self) -> Ordering {
        match self.base().cmp(&other.base()) {
            ord @ (Ordering::Less | Ordering::Greater) => ord,
            Ordering::Equal => self.len().cmp(&other.len()),
        }
    }
}

impl<Idx, V> PartialOrd for Mapping<Idx, V>
where
    Idx: Value,
    V: Entry,
{
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
