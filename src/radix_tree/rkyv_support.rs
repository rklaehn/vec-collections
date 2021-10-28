use std::{collections::{BTreeMap, BTreeSet}, sync::Arc};

use crate::AbstractRadixTreeMut;

use super::{lazy::Lazy, AbstractRadixTree, Fragment, RadixTree, TKey, TValue};
use rkyv::{
    option::ArchivedOption,
    ser::{ScratchSpace, Serializer, SharedSerializeRegistry},
    vec::{ArchivedVec, VecResolver},
    Archive, Archived, Deserialize, DeserializeUnsized, Fallible, Resolver, Serialize,
};

#[derive(Clone)]
pub struct LazyRadixTree<'a, K, V>
    where
        K: TKey,
        V: TValue,
{
    prefix: Fragment<K>,
    value: Option<V>,
    /// the children are lazy loaded at the time of first access.
    children: Lazy<&'a [ArchivedRadixTree2<K, V>], Arc<Vec<Self>>>,
}

impl<'a, K: TKey, V: TValue> Default for LazyRadixTree<'a, K, V> {
    fn default() -> Self {
        Self {
            prefix: Default::default(),
            value: Default::default(),
            children: Default::default(),
        }
    }
}

impl<'a, K: TKey, V: TValue> AbstractRadixTreeMut<K, V> for LazyRadixTree<'a, K, V> {
    fn new(prefix: Fragment<K>, value: Option<V>, children: Vec<Self>) -> Self {
        let children = Lazy::initialized(Arc::new(children));
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

impl<K: TKey, V: TValue> From<RadixTree<K, V>> for LazyRadixTree<'static, K, V> {
    fn from(value: RadixTree<K, V>) -> Self {
        let RadixTree {
            prefix,
            value,
            children,
        } = value;
        let children = children.into_iter().map(Self::from).collect::<Vec<_>>();
        let children = Lazy::initialized(Arc::new(children));
        Self {
            prefix,
            value,
            children,
        }
    }
}

impl<'a, K: TKey + Archive<Archived = K>, V: TValue + Archive<Archived = V>>
    From<&'a ArchivedRadixTree2<K, V>> for LazyRadixTree<'a, K, V>
{
    fn from(value: &'a ArchivedRadixTree2<K, V>) -> Self {
        let children = value.children().iter().map(Self::from).collect::<Vec<_>>();
        let children = Lazy::initialized(Arc::new(children));
        LazyRadixTree {
            prefix: value.prefix().into(),
            value: value.value().cloned(),
            children,
        }
    }
}

impl<'a, K: TKey, V: TValue> AbstractRadixTree<K, V> for LazyRadixTree<'a, K, V> {
    type Materialized = LazyRadixTree<'a, K, V>;

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

impl<K: TKey, V: TValue> AbstractRadixTree<K, V>
    for ArchivedRadixTree2<K, V>
{
    type Materialized = LazyRadixTree<'static, K, V>;

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

impl<'a, K: TKey, V: TValue> LazyRadixTree<'a, K, V>
{
    fn children_arc(&self) -> &Arc<Vec<Self>> {
        self.children.get_or_create(materialize_shallow)
    }

    fn children_arc_mut(&mut self) -> &mut Arc<Vec<Self>> {
        self.children.get_or_create_mut(materialize_shallow)
    }

    fn maybe_arc(&self) -> Option<&Arc<Vec<Self>>> {
        self.children.get()
    }

    pub fn all_arcs(&self, into: &mut BTreeMap<usize, Arc<Vec<Self>>>) {
        if let Some(children) = self.maybe_arc() {
            into.insert(location(children.as_ref()), children.clone());
            for child in children.iter() {
                child.all_arcs(into);
            }
        }
    }
}

fn materialize_shallow<K: TKey, V: TValue>(
    children: &[ArchivedRadixTree2<K, V>],
) -> Arc<Vec<LazyRadixTree<K, V>>> {
    Arc::new(
        children
            .iter()
            .map(|child| LazyRadixTree {
                prefix: child.prefix.as_ref().into(),
                value: child.value.as_ref().cloned(),
                children: Lazy::uninitialized(child.children.as_ref()),
            })
            .collect(),
    )
}

#[derive(Debug)]
#[repr(C)]
pub struct ArchivedRadixTree<K, V> {
    prefix: ArchivedVec<K>,
    value: ArchivedOption<V>,
    children: ArchivedVec<ArchivedRadixTree<K, V>>,
}

pub struct LazyRadixTreeResolver<K: TKey + Archive, V: TValue + Archive>
{
    prefix: Resolver<Vec<K>>,
    value: Resolver<Option<V>>,
    children: Resolver<Arc<Vec<LazyRadixTree<'static, K, V>>>>,
}

#[repr(C)]
pub struct ArchivedRadixTree2<K, V>
where
    K: TKey,
    V: TValue,
{
    prefix: Archived<Vec<K>>,
    value: Archived<Option<V>>,
    children: Archived<Arc<Vec<LazyRadixTree<'static, K, V>>>>,
}

impl<K: TKey, V: TValue> AbstractRadixTree<K, V> for ArchivedRadixTree<K, V> {
    fn prefix(&self) -> &[K] {
        &self.prefix
    }

    fn value(&self) -> Option<&V> {
        self.value.as_ref()
    }

    fn children(&self) -> &[Self] {
        &self.children
    }

    type Materialized = RadixTree<K, V>;
}

fn offset_from<T, U>(base: *const T, p: *const U) -> usize {
    let base = base as usize;
    let p = p as usize;
    assert!(p >= base);
    p - base
}

fn location<T>(x: &T) -> usize {
    (x as *const T) as usize
}

impl<K, V> Archive for RadixTree<K, V>
where
    K: TKey + Archive,
    V: TValue + Archive,
{
    type Archived = ArchivedRadixTree<K, V>;

    type Resolver = (VecResolver, Option<<V as Archive>::Resolver>, VecResolver);

    unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
        let (prefix, value, children) = resolver;
        let ptr = &mut (*out).prefix;
        ArchivedVec::resolve_from_slice(self.prefix(), pos + offset_from(out, ptr), prefix, ptr);
        let ptr = &mut (*out).value;
        self.value()
            .cloned()
            .resolve(pos + offset_from(out, ptr), value, ptr);
        let ptr = &mut (*out).children;
        ArchivedVec::resolve_from_slice(
            self.children(),
            pos + offset_from(out, ptr),
            children,
            ptr,
        );
    }
}

impl<'a, K: TKey, V: TValue> Archive for LazyRadixTree<'a, K, V>
{
    type Archived = ArchivedRadixTree2<K, V>;

    type Resolver = LazyRadixTreeResolver<K, V>;

    unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
        let LazyRadixTreeResolver {
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
        self.children_arc().resolve(pos + offset_from(out, ptr), children, ptr);
    }
}

impl<S, K, V> Serialize<S> for RadixTree<K, V>
where
    K: TKey + Serialize<S>,
    V: TValue + Serialize<S>,
    S: ScratchSpace + Serializer,
{
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        let prefix = ArchivedVec::serialize_from_slice(self.prefix(), serializer)?;
        let value = self.value().cloned().serialize(serializer)?;
        let children = ArchivedVec::serialize_from_slice(self.children(), serializer)?;
        Ok((prefix, value, children))
    }
}

impl<'a, S, K, V> Serialize<S> for LazyRadixTree<'a, K, V>
where
    K: TKey + Serialize<S>,
    V: TValue + Serialize<S>,
    S: ScratchSpace + Serializer + SharedSerializeRegistry,
{
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        let prefix = rkyv::vec::ArchivedVec::serialize_from_slice(self.prefix(), serializer)?;
        let value = self.value().cloned().serialize(serializer)?;
        let arc = self.children_arc();
        let arc: &Arc<Vec<LazyRadixTree<'static, K, V>>> = unsafe { std::mem::transmute(arc) };
        let children = arc.serialize(serializer)?;
        Ok(LazyRadixTreeResolver {
            prefix,
            value,
            children,
        })
    }
}

impl<D, K, V> Deserialize<RadixTree<K, V>, D> for ArchivedRadixTree<K, V>
where
    D: Fallible + ?Sized,
    K: TKey + Deserialize<K, D> + Clone,
    V: TValue + Deserialize<V, D>,
    [K]: DeserializeUnsized<[K], D>,
{
    fn deserialize(&self, deserializer: &mut D) -> Result<RadixTree<K, V>, D::Error> {
        let prefix: Vec<K> = self.prefix.deserialize(deserializer)?;
        let value: Option<V> = self.value.deserialize(deserializer)?;
        let children: Vec<RadixTree<K, V>> = self.children.deserialize(deserializer)?;
        Ok(RadixTree::new(
            Fragment::from(prefix.as_ref()),
            value,
            children,
        ))
    }
}

#[cfg(test)]
mod tests {
    use crate::{AbstractRadixTree, AbstractRadixTreeMut, RadixTree};

    fn mk_string(n: usize) -> String {
        let text = n.to_string();
        text.chars()
            .flat_map(|c| std::iter::repeat(c).take(100))
            .collect::<String>()
    }

    #[test]
    fn archive_smoke() {
        let mut a = RadixTree::empty();
        for i in 0..1000 {
            a.union_with(&RadixTree::single(mk_string(i).as_bytes(), ()));
        }
        use rkyv::*;
        use ser::Serializer;
        let mut serializer = ser::serializers::AllocSerializer::<256>::default();
        serializer.serialize_value(&a).unwrap();
        let bytes = serializer.into_serializer().into_inner();
        let archived = unsafe { rkyv::archived_root::<RadixTree<u8, ()>>(&bytes) };
        println!("size:    {}", bytes.len());
        println!(
            "key size:{}",
            archived
                .iter()
                .map(|(k, v)| k.as_ref().len())
                .sum::<usize>()
        );
        // hexdump::hexdump(&bytes);
        // println!("{:#?}", a);
        // println!("{}", hex::encode(&bytes));
        let result: RadixTree<u8, ()> = archived.deserialize(&mut Infallible).unwrap();
        // println!("{:#?}", result);
    }
}
