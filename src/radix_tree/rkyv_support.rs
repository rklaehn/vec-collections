use crate::AbstractRadixTreeMut;

use super::{lazy::Lazy, AbstractRadixTree, Fragment, RadixTree, TKey, TValue};
use rkyv::{
    option::ArchivedOption,
    ser::{ScratchSpace, Serializer},
    vec::{ArchivedVec, VecResolver},
    Archive, Deserialize, DeserializeUnsized, Fallible, Serialize,
};

#[derive(Clone)]
pub struct LazyRadixTree<'a, K, V> {
    prefix: Fragment<K>,
    value: Option<V>,
    /// the children are lazy loaded at the time of first access.
    children: Lazy<&'a [ArchivedRadixTree<K, V>], Vec<Self>>,
}

impl<'a, K, V> Default for LazyRadixTree<'a, K, V> {
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
        let children = Lazy::initialized(children);
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
        self.children.get_or_create_mut(materialize_shallow)
    }
}

impl<K, V> From<RadixTree<K, V>> for LazyRadixTree<'static, K, V> {
    fn from(value: RadixTree<K, V>) -> Self {
        let RadixTree {
            prefix,
            value,
            children,
        } = value;
        let children = children.into_iter().map(Self::from).collect::<Vec<_>>();
        let children = Lazy::initialized(children);
        Self {
            prefix,
            value,
            children,
        }
    }
}

impl<'a, K: TKey, V: TValue> From<&'a ArchivedRadixTree<K, V>> for LazyRadixTree<'a, K, V> {
    fn from(value: &'a ArchivedRadixTree<K, V>) -> Self {
        let children = value.children().iter().map(Self::from).collect::<Vec<_>>();
        let children = Lazy::initialized(children);
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
        self.children.get_or_create(materialize_shallow)
    }
}

fn materialize_shallow<K: TKey, V: TValue>(
    children: &[ArchivedRadixTree<K, V>],
) -> Vec<LazyRadixTree<'_, K, V>> {
    children
        .iter()
        .map(|child| LazyRadixTree {
            prefix: child.prefix.as_ref().into(),
            value: child.value.as_ref().cloned(),
            children: Lazy::uninitialized(child.children.as_ref()),
        })
        .collect()
}

#[repr(C)]
pub struct ArchivedRadixTree<K, V> {
    prefix: ArchivedVec<K>,
    value: ArchivedOption<V>,
    children: ArchivedVec<ArchivedRadixTree<K, V>>,
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

impl<K, V> Archive for RadixTree<K, V>
where
    K: TKey + Archive<Archived = K>,
    V: TValue + Archive<Archived = V>,
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

impl<'a, K, V> Archive for LazyRadixTree<'a, K, V>
where
    K: TKey + Archive<Archived = K> + 'static,
    V: TValue + Archive<Archived = V> + 'static,
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

impl<S, K, V> Serialize<S> for RadixTree<K, V>
where
    K: TKey + Archive<Archived = K> + Serialize<S>,
    V: TValue + Archive<Archived = V> + Serialize<S>,
    S: ScratchSpace + Serializer,
{
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        let prefix = rkyv::vec::ArchivedVec::serialize_from_slice(self.prefix(), serializer)?;
        let value = self.value().cloned().serialize(serializer)?;
        let children = rkyv::vec::ArchivedVec::serialize_from_slice(self.children(), serializer)?;
        Ok((prefix, value, children))
    }
}

impl<'a, S, K, V> Serialize<S> for LazyRadixTree<'a, K, V>
where
    K: TKey + Archive<Archived = K> + Serialize<S> + 'static,
    V: TValue + Archive<Archived = V> + Serialize<S> + 'static,
    S: ScratchSpace + Serializer,
{
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        let prefix = rkyv::vec::ArchivedVec::serialize_from_slice(self.prefix(), serializer)?;
        let value = self.value().cloned().serialize(serializer)?;
        let children = rkyv::vec::ArchivedVec::serialize_from_slice(self.children(), serializer)?;
        Ok((prefix, value, children))
    }
}

impl<D, K, V> Deserialize<RadixTree<K, V>, D> for ArchivedRadixTree<K, V>
where
    D: Fallible + ?Sized,
    K: TKey + Archive<Archived = K> + Deserialize<K, D> + Clone,
    V: TValue + Archive<Archived = V> + Deserialize<V, D>,
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
