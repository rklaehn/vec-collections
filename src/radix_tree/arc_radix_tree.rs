use std::{
    collections::BTreeMap,
    sync::{Arc, LazyLock},
};

use internals::{AbstractRadixTreeMut as _, Fragment};
use rkyv::{
    de::SharedDeserializeRegistry,
    ser::{ScratchSpace, Serializer, SharedSerializeRegistry},
    vec::ArchivedVec,
    Archive, Archived, Deserialize, Resolver, Serialize,
};

use super::{internals, location, offset_from, AbstractRadixTree, RadixTree, TKey, TValue};

static EMPTY_ARC_VEC: LazyLock<Arc<Vec<u128>>> = LazyLock::new(|| Arc::new(Vec::new()));

fn empty_arc<T>() -> Arc<Vec<T>> {
    // TODO: this seems to work, but is strictly speaking ub. Get rid of the unsafe
    unsafe { std::mem::transmute(EMPTY_ARC_VEC.clone()) }
}

fn wrap_in_arc<T>(data: Vec<T>) -> Arc<Vec<T>> {
    if data.is_empty() {
        empty_arc()
    } else {
        Arc::new(data)
    }
}

/// A generic radix tree with structural sharing and copy on write
///
/// Compared to the simplest radix tree, this adds cheap (O(1)) snapshots and enables incremental
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
            children: empty_arc(),
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

impl<K: TKey, V: TValue> internals::AbstractRadixTreeMut<K, V> for ArcRadixTree<K, V> {
    fn new(prefix: Fragment<K>, value: Option<V>, children: Vec<Self>) -> Self {
        let children = wrap_in_arc(children);
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
        // this is what makes the data structure copy on write.
        // If we are the sole owner, this will not allocate and be very cheap.
        // if there is another owner (e.g. an old snapshot), this will clone the array.
        //
        // cloning will shrink to fit
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
        Self::new(prefix, value, children)
    }
}

impl<K: TKey, V: TValue> ArcRadixTree<K, V> {
    fn children_arc(&self) -> &Arc<Vec<Self>> {
        &self.children
    }

    fn children_arc_mut(&mut self) -> &mut Arc<Vec<Self>> {
        &mut self.children
    }

    /// copy all arcs that are used internally in this tree, and store them in a BTreeMap
    ///
    /// as long as the BTreeMap exists, this will have the effect of disabling reuse for
    /// all tree nodes and force copies on modification.
    pub fn all_arcs(&self, into: &mut BTreeMap<usize, Arc<Vec<Self>>>) {
        let children = self.children_arc();
        into.insert(location(children.as_ref()), children.clone());
        for child in children.iter() {
            child.all_arcs(into);
        }
    }
}

impl<K: TKey, V: TValue + Archive<Archived = V>> From<&ArchivedArcRadixTree<K, V>>
    for ArcRadixTree<K, V>
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

impl<K: TKey, V: TValue + Archive<Archived = V>> AbstractRadixTree<K, V>
    for ArchivedArcRadixTree<K, V>
{
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

pub struct ArcRadixTreeResolver<K: TKey, V: TValue> {
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
    K: TKey,
    V: TValue,
    Archived<K>: Deserialize<K, D>,
    Archived<V>: Deserialize<V, D>,
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

#[cfg(feature = "rkyv_validated")]
mod validation_support {
    use core::fmt;

    use bytecheck::CheckBytes;
    use rkyv::{
        validation::{ArchiveContext, SharedContext},
        Archived,
    };

    use super::{ArchivedArcRadixTree, TKey, TValue};

    /// Validation error for a radix tree
    #[derive(Debug)]
    pub enum ArchivedRadixTreeError {
        /// error with the prefix
        Prefix,
        /// error with the value
        Value,
        /// error with the children
        Children(String),
        /// error with the order of the children
        Order,
    }

    impl std::error::Error for ArchivedRadixTreeError {}

    impl std::fmt::Display for ArchivedRadixTreeError {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{:?}", self)
        }
    }
    impl<C, K, V> bytecheck::CheckBytes<C> for ArchivedArcRadixTree<K, V>
    where
        C: ?Sized + ArchiveContext + SharedContext,
        C::Error: std::error::Error,
        K: TKey,
        V: TValue,
        Archived<Vec<K>>: bytecheck::CheckBytes<C>,
        Archived<Option<V>>: bytecheck::CheckBytes<C>,
    {
        type Error = ArchivedRadixTreeError;
        unsafe fn check_bytes<'a>(
            this: *const Self,
            context: &mut C,
        ) -> Result<&'a Self, Self::Error> {
            let Self {
                prefix,
                value,
                children,
            } = &(*this);
            // check the prefix
            CheckBytes::check_bytes(prefix, context).map_err(|_| ArchivedRadixTreeError::Prefix)?;
            // check the value, if present
            CheckBytes::check_bytes(value, context).map_err(|_| ArchivedRadixTreeError::Value)?;
            // check that the prefix of all children is of non zero length
            if !children.iter().all(|child| !child.prefix.is_empty()) {
                return Err(ArchivedRadixTreeError::Children(
                    "empty child prefix".into(),
                ));
            };
            // check the order of the children
            if !children
                .iter()
                .zip(children.iter().skip(1))
                .all(|(a, b)| a.prefix[0] < b.prefix[0])
            {
                return Err(ArchivedRadixTreeError::Order);
            };
            // recursively check the children
            CheckBytes::check_bytes(children, context)
                .map_err(|e| ArchivedRadixTreeError::Children(e.to_string()))?;

            Ok(&*this)
        }
    }
}
