use num_traits::Zero;
use serde::de::Deserialize;
use serde::de::DeserializeOwned;
use serde::de::Deserializer;
use serde::ser::{Serialize, Serializer};
use sorted_iter::*;
use std::collections::BTreeMap;
use std::ops::{Add, Sub};

#[derive(Clone, Debug)]
pub struct TagTree<K: Ord, V> {
    value: Option<V>,
    children: BTreeMap<K, Self>,
}

impl<K: Ord + Clone + Serialize, V: Clone + Serialize> Serialize for TagTree<K, V> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let x: SerdeHelper<K, V> = self.clone().into();
        x.serialize(serializer)
    }
}

impl<'de, K: Ord, V> Deserialize<'de> for TagTree<K, V> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let x: SerdeHelper<K, V> = SerdeHelper::<K, V>::deserialize(deserializer)?;
        Ok(x.into())
    }
}

enum SerdeHelper<K: Ord, V> {
    Empty,
    Value((V,)),
    Children(BTreeMap<K, TagTree<K, V>>),
    Both((V, BTreeMap<K, TagTree<K, V>>)),
}

impl<K: Ord + Clone + Serialize, V: Clone + Serialize> Serialize for SerdeHelper<K, V> {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            SerdeHelper::Empty => ().serialize(serializer),
            SerdeHelper::Value(x) => x.serialize(serializer),
            SerdeHelper::Children(x) => x.serialize(serializer),
            SerdeHelper::Both(x) => x.serialize(serializer),
        }
    }
}

impl<'de, K: Ord, V> Deserialize<'de> for SerdeHelper<K, V> {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        unimplemented!()
    }
}

impl<K: Ord, V> From<TagTree<K, V>> for SerdeHelper<K, V> {
    fn from(value: TagTree<K, V>) -> Self {
        if let Some(v) = value.value {
            if value.children.is_empty() {
                Self::Value((v,))
            } else {
                Self::Both((v, value.children))
            }
        } else {
            if value.children.is_empty() {
                Self::Empty
            } else {
                Self::Children(value.children)
            }
        }
    }
}

impl<K: Ord, V> From<SerdeHelper<K, V>> for TagTree<K, V> {
    fn from(value: SerdeHelper<K, V>) -> Self {
        match value {
            SerdeHelper::Empty => Self::default(),
            SerdeHelper::Value((v,)) => Self::new(Some(v), Default::default()),
            SerdeHelper::Children(c) => Self::new(Default::default(), c),
            SerdeHelper::Both((v, c)) => Self::new(Some(v), c),
        }
    }
}

impl<K: Ord, V> Default for TagTree<K, V> {
    fn default() -> Self {
        Self::empty()
    }
}

impl<K: Ord, V> TagTree<K, V> {
    pub fn empty() -> Self {
        Self {
            value: Default::default(),
            children: Default::default(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.value.is_none() && self.children.is_empty()
    }

    fn new(value: Option<V>, children: BTreeMap<K, Self>) -> Self {
        Self { value, children }
    }

    fn non_empty(value: Option<V>, children: BTreeMap<K, Self>) -> Option<Self> {
        let res = Self::new(value, children);
        if res.is_empty() {
            None
        } else {
            Some(res)
        }
    }

    fn leaf(value: V) -> Self {
        Self::new(Some(value), BTreeMap::new())
    }

    fn single(key: Vec<K>, value: V) -> Self {
        let leaf = Self::new(Some(value), BTreeMap::new());
        let mut result = leaf;
        for part in key.into_iter().rev() {
            result = Self::new(None, std::iter::once((part, result)).collect())
        }
        result
    }

    fn filter_map_values<W>(self, f: &impl Fn(V) -> Option<W>) -> Option<TagTree<K, W>> {
        let value = self.value.and_then(f);
        let children: BTreeMap<K, TagTree<K, W>> = self
            .children
            .into_iter()
            .filter_map_values(|child| child.filter_map_values(&f))
            .collect();
        TagTree::<K, W>::non_empty(value, children)
    }
}

impl<K: Ord + Clone, V: Clone> TagTree<K, V> {
    fn combine_values(a: &Option<V>, b: &Option<V>, f: &impl Fn(&V, &V) -> Option<V>) -> Option<V> {
        match (a, b) {
            (None, None) => None,
            (a, None) => a.clone(),
            (None, b) => b.clone(),
            (Some(a), Some(b)) => f(a, b),
        }
    }

    fn combine_children(
        a: Option<&Self>,
        b: Option<&Self>,
        f: &impl Fn(&V, &V) -> Option<V>,
    ) -> Option<Self> {
        match (a, b) {
            (None, None) => None,
            (a, None) => a.cloned(),
            (None, b) => b.cloned(),
            (Some(a), Some(b)) => a.outer_join_with(b, f),
        }
    }

    fn outer_join_with(&self, that: &Self, f: &impl Fn(&V, &V) -> Option<V>) -> Option<Self> {
        let value = Self::combine_values(&self.value, &that.value, f);
        let children: BTreeMap<K, Self> = self
            .children
            .iter()
            .outer_join(that.children.iter())
            .filter_map_values(|(a, b)| Self::combine_children(a, b, f))
            .map(|(k, v)| (k.clone(), v))
            .collect();
        Self::non_empty(value, children)
    }

    fn inner_join_with(&self, that: &Self, f: &impl Fn(&V, &V) -> Option<V>) -> Option<Self> {
        let value = Self::combine_values(&self.value, &that.value, f);
        let children: BTreeMap<K, Self> = self
            .children
            .iter()
            .join(that.children.iter())
            .filter_map_values(|(a, b)| a.inner_join_with(b, f))
            .map(|(k, v)| (k.clone(), v))
            .collect();
        Self::non_empty(value, children)
    }
}

fn to_option<R: Zero>(value: R) -> Option<R> {
    if value.is_zero() {
        None
    } else {
        Some(value)
    }
}

impl<K: Ord + Clone, V: Ord + Clone + Zero> TagTree<K, V> {
    fn min(&self, that: &Self) -> Self {
        let f = |a: &V, b: &V| to_option(std::cmp::min(a, b).clone());
        self.inner_join_with(that, &f).unwrap_or_default()
    }

    fn max(&self, that: &Self) -> Self {
        let f = |a: &V, b: &V| to_option(std::cmp::max(a, b).clone());
        self.inner_join_with(that, &f).unwrap_or_default()
    }
}

impl<K: Ord + Clone, V: Add<Output = V> + Zero + Clone> Add for &TagTree<K, V> {
    type Output = TagTree<K, V>;
    fn add(self, that: Self) -> Self::Output {
        let f = |a: &V, b: &V| to_option(a.clone() + b.clone());
        self.outer_join_with(that, &f).unwrap_or_default()
    }
}

impl<K: Ord + Clone, V: Sub<Output = V> + Zero + Clone> Sub for &TagTree<K, V> {
    type Output = TagTree<K, V>;
    fn sub(self, that: Self) -> Self::Output {
        let f = |a: &V, b: &V| to_option(a.clone() - b.clone());
        self.outer_join_with(that, &f).unwrap_or_default()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn smoke_test() {
        let a: TagTree<&str, i32> = TagTree::single(vec!["a"], 1);
        let b: TagTree<&str, i32> = TagTree::single(vec!["b"], 2);
        let r = &a + &b;
        let x = r.min(&b).is_empty();
        println!("{:?} {:?}", r, x);
        println!("{}", serde_json::to_string_pretty(&r).unwrap());
    }
}
