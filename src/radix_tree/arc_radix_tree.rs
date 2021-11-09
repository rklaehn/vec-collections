use crate::AbstractRadixTreeMut;
use std::{collections::BTreeMap, sync::Arc};

use super::{
    location, offset_from, AbstractRadixTree, Fragment, RadixTree, TKey, TValue,
};
use rkyv::{Archive, Archived, Deserialize, Resolver, Serialize, de::{SharedDeserializeRegistry}, ser::{ScratchSpace, Serializer, SharedSerializeRegistry}, vec::ArchivedVec};

#[derive(Clone)]
pub struct ArcRadixTree<K, V>
where
    K: TKey,
    V: TValue,
{
    prefix: Fragment<K>,
    value: Option<V>,
    children: Arc<Vec<Self>>,
}

impl<K: TKey, V: TValue> Default for ArcRadixTree<K, V> {
    fn default() -> Self {
        Self {
            prefix: Default::default(),
            value: Default::default(),
            children: Default::default(),
        }
    }
}

impl<K: TKey, V: TValue> AbstractRadixTree<K, V> for ArcRadixTree<K, V> {
    type Materialized = ArcRadixTree<K, V>;

    fn prefix(&self) -> &[K] {
        &self.prefix
    }

    fn value(&self) -> Option<&V> {
        self.value.as_ref()
    }

    fn children(&self) -> &[Self] {
        self.children_arc().as_ref()
    }
}

impl<K: TKey, V: TValue> AbstractRadixTreeMut<K, V> for ArcRadixTree<K, V> {
    fn new(prefix: Fragment<K>, value: Option<V>, children: Vec<Self>) -> Self {
        let children = Arc::new(children);
        Self {
            prefix,
            value,
            children,
        }
    }

    fn value_mut(&mut self) -> &mut Option<V> {
        &mut self.value
    }

    fn prefix_mut(&mut self) -> &mut Fragment<K> {
        &mut self.prefix
    }

    fn children_mut(&mut self) -> &mut Vec<Self> {
        Arc::make_mut(self.children_arc_mut())
    }
}

impl<K: TKey, V: TValue> From<RadixTree<K, V>> for ArcRadixTree<K, V> {
    fn from(value: RadixTree<K, V>) -> Self {
        let RadixTree {
            prefix,
            value,
            children,
        } = value;
        let children = children.into_iter().map(Self::from).collect::<Vec<_>>();        
        Self::new(
            prefix,
            value,
            children,
        )
    }
}

impl<K: TKey, V: TValue> ArcRadixTree<K, V> {
    fn children_arc(&self) -> &Arc<Vec<Self>> {
        &self.children
    }

    fn children_arc_mut(&mut self) -> &mut Arc<Vec<Self>> {
        &mut self.children
    }

    pub fn all_arcs(&self, into: &mut BTreeMap<usize, Arc<Vec<Self>>>) {
        let children = self.children_arc();
        into.insert(location(children.as_ref()), children.clone());
        for child in children.iter() {
            child.all_arcs(into);
        }
    }
}

impl<K: TKey + Archive<Archived = K>, V: TValue + Archive<Archived = V>>
    From<&ArchivedArcRadixTree<K, V>> for ArcRadixTree<K, V>
{
    fn from(value: &ArchivedArcRadixTree<K, V>) -> Self {
        let children = value.children().iter().map(Self::from).collect::<Vec<_>>();
        let children = Arc::new(children);
        ArcRadixTree {
            prefix: value.prefix().into(),
            value: value.value().cloned(),
            children,
        }
    }
}

impl<K: TKey, V: TValue> AbstractRadixTree<K, V> for ArchivedArcRadixTree<K, V> {
    type Materialized = ArcRadixTree<K, V>;

    fn prefix(&self) -> &[K] {
        &self.prefix
    }

    fn value(&self) -> Option<&V> {
        self.value.as_ref()
    }

    fn children(&self) -> &[Self] {
        &self.children
    }
}

pub struct ArcRadixTreeResolver<K: TKey + Archive, V: TValue + Archive> {
    prefix: Resolver<Vec<K>>,
    value: Resolver<Option<V>>,
    children: Resolver<Arc<Vec<ArcRadixTree<K, V>>>>,
}

#[repr(C)]
pub struct ArchivedArcRadixTree<K, V>
where
    K: TKey,
    V: TValue,
{
    prefix: Archived<Vec<K>>,
    value: Archived<Option<V>>,
    children: Archived<Arc<Vec<ArcRadixTree<K, V>>>>,
}

impl<'a, K: TKey, V: TValue> Archive for ArcRadixTree<K, V> {
    type Archived = ArchivedArcRadixTree<K, V>;

    type Resolver = ArcRadixTreeResolver<K, V>;

    unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
        let ArcRadixTreeResolver {
            prefix,
            value,
            children,
        } = resolver;
        let ptr = &mut (*out).prefix;
        ArchivedVec::resolve_from_slice(self.prefix(), pos + offset_from(out, ptr), prefix, ptr);
        let ptr = &mut (*out).value;
        self.value()
            .cloned()
            .resolve(pos + offset_from(out, ptr), value, ptr);
        let ptr = &mut (*out).children;
        self.children_arc()
            .resolve(pos + offset_from(out, ptr), children, ptr);
    }
}

impl<S, K, V> Serialize<S> for ArcRadixTree<K, V>
where
    K: TKey + Serialize<S>,
    V: TValue + Serialize<S>,
    S: ScratchSpace + Serializer + SharedSerializeRegistry,
{
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        let prefix = rkyv::vec::ArchivedVec::serialize_from_slice(self.prefix(), serializer)?;
        let value = self.value().cloned().serialize(serializer)?;
        let arc = self.children_arc();
        let arc: &Arc<Vec<ArcRadixTree<K, V>>> = unsafe { std::mem::transmute(arc) };
        let children = arc.serialize(serializer)?;
        Ok(ArcRadixTreeResolver {
            prefix,
            value,
            children,
        })
    }
}

impl<D, K, V> Deserialize<ArcRadixTree<K, V>, D> for ArchivedArcRadixTree<K, V>
where
    D: SharedDeserializeRegistry,
    K: TKey + Deserialize<K, D>,
    V: TValue + Deserialize<V, D>,
{
    fn deserialize(&self, deserializer: &mut D) -> Result<ArcRadixTree<K, V>, D::Error> {
        let prefix: Vec<K> = self.prefix.deserialize(deserializer)?;
        let value: Option<V> = self.value.deserialize(deserializer)?;
        let children: Arc<Vec<ArcRadixTree<K, V>>> = self.children.deserialize(deserializer)?;
        Ok(ArcRadixTree {
            prefix: Fragment::from(prefix.as_ref()),
            value,
            children,
        })
    }
}