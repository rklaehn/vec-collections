use super::{AbstractRadixTree, Fragment, RadixTree};
use lazy_init::LazyTransform;
use rkyv::{option::ArchivedOption, vec::ArchivedVec, DeserializeUnsized};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LazyRadixTree<'a, K, V> {
    prefix: Fragment<K>,
    value: Option<V>,
    /// the children are lazy loaded at the time of first access.
    children: LazyTransform<&'a [ArchivedRadixTree<K, V>], Vec<Self>>,
}

impl<K, V> From<RadixTree<K, V>> for LazyRadixTree<'static, K, V> {
    fn from(value: RadixTree<K, V>) -> Self {
        let RadixTree {
            prefix,
            value,
            children,
        } = value;
        let m = children.into_iter().map(Self::from).collect::<Vec<_>>();
        let children = LazyTransform::<&'static [ArchivedRadixTree<K, V>], Vec<Self>>::new(&[]);
        children.get_or_create(|_| m);
        Self {
            prefix,
            value,
            children,
        }
    }
}

impl<'a, K: Clone, V: Clone> AbstractRadixTree<K, V> for LazyRadixTree<'a, K, V> {
    fn prefix(&self) -> &[K] {
        &self.prefix
    }

    fn value(&self) -> Option<&V> {
        self.value.as_ref()
    }

    fn children(&self) -> &[Self] {
        self.children.get_or_create(|children| {
            children
                .iter()
                .map(|child| Self {
                    prefix: child.prefix.as_ref().into(),
                    value: child.value.as_ref().cloned(),
                    children: LazyTransform::new(child.children.as_ref()),
                })
                .collect()
        })
    }
}

#[repr(C)]
pub struct ArchivedRadixTree<K, V> {
    prefix: ArchivedVec<K>,
    value: ArchivedOption<V>,
    children: ArchivedVec<ArchivedRadixTree<K, V>>,
}

impl<K, V> AbstractRadixTree<K, V> for ArchivedRadixTree<K, V> {
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

fn offset_from<T, U>(base: *const T, p: *const U) -> usize {
    let base = base as usize;
    let p = p as usize;
    assert!(p >= base);
    p - base
}

impl<K, V> rkyv::Archive for RadixTree<K, V>
where
    K: rkyv::Archive<Archived = K>,
    V: rkyv::Archive<Archived = V>,
{
    type Archived = ArchivedRadixTree<K, V>;

    type Resolver = (
        rkyv::vec::VecResolver,
        Option<<V as rkyv::Archive>::Resolver>,
        rkyv::vec::VecResolver,
    );

    unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
        let (prefix, value, children) = resolver;
        let ptr = &mut (*out).prefix;
        rkyv::vec::ArchivedVec::resolve_from_slice(
            self.prefix(),
            pos + offset_from(out, ptr),
            prefix,
            ptr,
        );
        let ptr = &mut (*out).value;
        self.value()
            .resolve(pos + offset_from(out, ptr), value, ptr);
        let ptr = &mut (*out).children;
        rkyv::vec::ArchivedVec::resolve_from_slice(
            self.children(),
            pos + offset_from(out, ptr),
            children,
            ptr,
        );
    }
}

impl<S, K, V> rkyv::Serialize<S> for RadixTree<K, V>
where
    K: rkyv::Archive<Archived = K> + rkyv::Serialize<S>,
    V: rkyv::Archive<Archived = V> + rkyv::Serialize<S>,
    S: rkyv::ser::ScratchSpace + rkyv::ser::Serializer,
{
    fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
        let prefix = rkyv::vec::ArchivedVec::serialize_from_slice(self.prefix(), serializer)?;
        let value = self.value().serialize(serializer)?;
        let children = rkyv::vec::ArchivedVec::serialize_from_slice(self.children(), serializer)?;
        Ok((prefix, value, children))
    }
}

impl<D, K, V> rkyv::Deserialize<RadixTree<K, V>, D> for ArchivedRadixTree<K, V>
where
    D: rkyv::Fallible + ?Sized,
    K: rkyv::Archive<Archived = K> + rkyv::Deserialize<K, D> + Clone,
    V: rkyv::Archive<Archived = V> + rkyv::Deserialize<V, D>,
    [K]: DeserializeUnsized<[K], D>,
{
    fn deserialize(&self, deserializer: &mut D) -> Result<RadixTree<K, V>, D::Error> {
        let prefix: Vec<K> = self.prefix.deserialize(deserializer)?;
        let value: Option<V> = self.value.deserialize(deserializer)?;
        let children: Vec<RadixTree<K, V>> = self.children.deserialize(deserializer)?;
        Ok(RadixTree {
            prefix: Fragment::from(prefix.as_ref()),
            value,
            children,
        })
    }
}

#[cfg(test)]
mod tests {
    use crate::{AbstractRadixTree, RadixTree};

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
