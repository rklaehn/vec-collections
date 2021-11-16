use super::{internals, AbstractRadixTree, AbstractRadixTreeMut, Fragment, TKey, TValue};
use std::fmt::Debug;
use std::iter::FromIterator;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RadixTree<K, V> {
    pub(crate) prefix: Fragment<K>,
    pub(crate) value: Option<V>,
    pub(crate) children: Vec<Self>,
}

impl<K: TKey, V: TValue> AbstractRadixTree<K, V> for RadixTree<K, V> {
    type Materialized = RadixTree<K, V>;

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

impl<E: TKey, K: AsRef<[E]>, V: TValue> FromIterator<(K, V)> for RadixTree<E, V> {
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        let mut res = RadixTree::default();
        for (k, v) in iter.into_iter() {
            res.union_with(&RadixTree::single(k.as_ref(), v))
        }
        res
    }
}

impl<K: TKey, V: TValue> internals::AbstractRadixTreeMut<K, V> for RadixTree<K, V> {
    fn new(prefix: Fragment<K>, value: Option<V>, children: Vec<Self>) -> Self {
        Self {
            prefix,
            value,
            children,
        }
    }

    fn value_mut(&mut self) -> &mut Option<V> {
        &mut self.value
    }

    fn children_mut(&mut self) -> &mut Vec<Self> {
        &mut self.children
    }

    fn prefix_mut(&mut self) -> &mut Fragment<K> {
        &mut self.prefix
    }
}

impl<K: Clone, V> Default for RadixTree<K, V> {
    fn default() -> Self {
        Self {
            prefix: Fragment::default(),
            value: None,
            children: Vec::new(),
        }
    }
}

impl<K: Ord + Copy + Debug, V: Debug> RadixTree<K, V> {
    pub fn empty() -> Self {
        Self::default()
    }
}

#[cfg(feature = "rkyv")]
mod rkyv_support {
    use super::super::{internals, offset_from};
    use super::{AbstractRadixTree, Fragment, RadixTree, TKey, TValue};
    use internals::AbstractRadixTreeMut as _;
    use rkyv::{
        ser::{ScratchSpace, Serializer},
        vec::ArchivedVec,
        Archive, Archived, Deserialize, Fallible, Resolver, Serialize,
    };

    #[repr(C)]
    pub struct ArchivedRadixTree<K: TKey, V: TValue> {
        prefix: Archived<Vec<K>>,
        value: Archived<Option<V>>,
        children: Archived<Vec<RadixTree<K, V>>>,
    }

    impl<K: TKey, V: TValue> std::fmt::Debug for ArchivedRadixTree<K, V> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            f.debug_struct("ArchivedRadixTree").finish_non_exhaustive()
        }
    }

    pub struct RadixTreeResolver<K: TKey, V: TValue> {
        prefix: Resolver<Vec<K>>,
        value: Resolver<Option<V>>,
        children: Resolver<Vec<RadixTree<K, V>>>,
    }

    impl<K: TKey, V: TValue + Archive<Archived = V>> AbstractRadixTree<K, V>
        for ArchivedRadixTree<K, V>
    {
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

    impl<K, V> Archive for RadixTree<K, V>
    where
        K: TKey + Archive,
        V: TValue + Archive,
    {
        type Archived = ArchivedRadixTree<K, V>;

        type Resolver = RadixTreeResolver<K, V>;

        unsafe fn resolve(&self, pos: usize, resolver: Self::Resolver, out: *mut Self::Archived) {
            let RadixTreeResolver {
                prefix,
                value,
                children,
            } = resolver;
            let ptr = &mut (*out).prefix;
            ArchivedVec::resolve_from_slice(
                self.prefix(),
                pos + offset_from(out, ptr),
                prefix,
                ptr,
            );
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
        K: TKey + Serialize<S>,
        V: TValue + Serialize<S>,
        S: ScratchSpace + Serializer,
    {
        fn serialize(&self, serializer: &mut S) -> Result<Self::Resolver, S::Error> {
            let prefix = ArchivedVec::serialize_from_slice(self.prefix(), serializer)?;
            let value = self.value().cloned().serialize(serializer)?;
            let children = ArchivedVec::serialize_from_slice(self.children(), serializer)?;
            Ok(RadixTreeResolver {
                prefix,
                value,
                children,
            })
        }
    }

    impl<D, K, V> Deserialize<RadixTree<K, V>, D> for ArchivedRadixTree<K, V>
    where
        D: Fallible + ?Sized,
        K: TKey,
        V: TValue,
        Archived<K>: Deserialize<K, D>,
        Archived<V>: Deserialize<V, D>,
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

    #[cfg(feature = "rkyv_validated")]
    mod validation_support {
        use super::{TKey, TValue};
        use bytecheck::CheckBytes;
        use core::fmt;
        use rkyv::{validation::ArchiveContext, vec::ArchivedVec, Archived};

        use super::ArchivedRadixTree;

        /// Validation error for a range set
        #[derive(Debug)]
        pub enum ArchivedRadixTreeError {
            /// error with the prefix
            PrefixCheckError,
            /// error with the value
            ValueCheckError,
            /// error with the children
            ChildrenCheckError,
            /// error with the order of the children
            OrderCheckError,
        }

        impl std::error::Error for ArchivedRadixTreeError {}

        impl std::fmt::Display for ArchivedRadixTreeError {
            fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
                write!(f, "{:?}", self)
            }
        }
        impl<C, K, V> bytecheck::CheckBytes<C> for ArchivedRadixTree<K, V>
        where
            C: ?Sized + ArchiveContext,
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
                CheckBytes::check_bytes(prefix, context)
                    .map_err(|_| ArchivedRadixTreeError::PrefixCheckError)?;
                // check the value, if present
                CheckBytes::check_bytes(value, context)
                    .map_err(|_| ArchivedRadixTreeError::ValueCheckError)?;
                // check that the prefix of all children is of non zero length
                if !children.iter().all(|child| child.prefix.len() > 0) {
                    return Err(ArchivedRadixTreeError::ChildrenCheckError);
                };
                // check the order of the children
                if !children
                    .iter()
                    .zip(children.iter().skip(1))
                    .all(|(a, b)| a.prefix[0] < b.prefix[0])
                {
                    return Err(ArchivedRadixTreeError::OrderCheckError);
                };
                // recursively check the children
                CheckBytes::check_bytes(children, context)
                    .map_err(|_| ArchivedRadixTreeError::ChildrenCheckError)?;

                Ok(&*this)
            }
        }
    }
}

#[cfg(feature = "rkyv")]
#[cfg(test)]
mod tests {
    use super::{AbstractRadixTree, AbstractRadixTreeMut, RadixTree};

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
                .map(|(k, _)| k.as_ref().len())
                .sum::<usize>()
        );
        // hexdump::hexdump(&bytes);
        // println!("{:#?}", a);
        // println!("{}", hex::encode(&bytes));
        let _result: RadixTree<u8, ()> = archived.deserialize(&mut Infallible).unwrap();
        // println!("{:#?}", result);
    }
}
