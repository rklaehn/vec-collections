use crate::Fragment;
use rkyv::{option::ArchivedOption, vec::ArchivedVec, Archived, DeserializeUnsized};
use smallvec::SmallVec;

use super::{AbstractRadixTree, RadixTree};

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

    fn clone_shortened(&self, n: usize) -> RadixTree<K, V>
    where
        K: Clone,
        V: Clone,
    {
        assert!(n < self.prefix().len());
        RadixTree {
            prefix: self.prefix()[n..].into(),
            value: self.value().cloned(),
            children: self
                .children()
                .iter()
                .map(|x| x.clone_shortened(0))
                .collect(),
        }
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

    #[test]
    fn archive_smoke() {
        let mut a = RadixTree::empty();
        for i in 0..100 {
            a.union_with(&RadixTree::single(i.to_string().as_bytes(), ()));
        }
        use rkyv::*;
        use ser::Serializer;
        let mut serializer = ser::serializers::AllocSerializer::<256>::default();
        serializer.serialize_value(&a).unwrap();
        let bytes = serializer.into_serializer().into_inner();
        let archived = unsafe { rkyv::archived_root::<RadixTree<u8, ()>>(&bytes) };
        hexdump::hexdump(&bytes);
        println!("{:#?}\n{}", a, hex::encode(&bytes));
        println!(
            "archived root {:?}\n{:?}\n{:?}",
            archived.prefix(),
            archived.value(),
            archived.children().len()
        );
        println!(
            "archived child 0 {:?}\n{:?}\n{:?}",
            archived.children()[0].prefix(),
            archived.children()[0].value(),
            archived.children()[0].children().len()
        );
        println!(
            "archived child 1 {:?}\n{:?}\n{:?}",
            archived.children()[1].prefix(),
            archived.children()[1].value(),
            archived.children()[1].children().len()
        );
        let result: RadixTree<u8, ()> = archived.deserialize(&mut Infallible).unwrap();
        println!("{:#?}", result);
    }
}
