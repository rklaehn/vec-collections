use std::{collections::BTreeMap, sync::Arc};

use rkyv::{AlignedVec, Archive, Serialize, archived_root, ser::serializers::{AlignedSerializer, CompositeSerializer}, ser::{Serializer, serializers::{AllocScratch, FallbackScratch, HeapScratch, SharedSerializeMap}}};
use vec_collections::{AbstractRadixTree, AbstractRadixTreeMut, ArchivedRadixTree2, LazyRadixTree, TKey, TValue};

trait RadixDb<'a, K: TKey, V: TValue> {
    fn tree(&self) -> &LazyRadixTree<'a, K, V>;
    fn tree_mut(&mut self) -> &mut LazyRadixTree<'a, K, V>;
    fn flush(&mut self) -> anyhow::Result<()>;
}

struct InMemRadixDb<'a, K: TKey, V: TValue> {
    file: AlignedVec,
    map: Option<(SharedSerializeMap, BTreeMap<usize, Arc<Vec<LazyRadixTree<'a, K, V>>>>)>,
    tree: LazyRadixTree<'a, K, V>,
}

impl<'a, K: TKey, V: TValue> Default for InMemRadixDb<'a, K, V>
{
    fn default() -> Self {
        Self {
            file: Default::default(),
            map: Default::default(),
            tree: Default::default(),
        }
    }
}

impl<'a, K: TKey, V: TValue> InMemRadixDb<'a, K, V> {
    pub fn load(bytes: &[u8]) -> anyhow::Result<Self>
        where
            K: for<'x> Serialize<MySerializer<'x>>,
            V: for<'x> Serialize<MySerializer<'x>>,
    {
        // this is a lie - bytes does not really live for 'a
        let bytes: &'a [u8] = unsafe { std::mem::transmute(bytes) };
        let tree: &'a ArchivedRadixTree2<K, V> = unsafe { archived_root::<LazyRadixTree<K, V>>(bytes) };
        let tree: LazyRadixTree<'a, K, V> = LazyRadixTree::from(tree);
        let mut file = AlignedVec::new();
        let mut serializer = CompositeSerializer::new(
            AlignedSerializer::new(&mut file),
            Default::default(),
            Default::default(),
        );
        // this makes the lie true, after serialization with an empty SharedSerializerMap, the tree is completely self-contained
        serializer
            .serialize_value(&tree)
            .map_err(|e| anyhow::anyhow!("Error while serializing: {}", e))?;
        let (_, _, map) = serializer.into_components();
        let mut arcs = BTreeMap::default();
        tree.all_arcs(&mut arcs);
        Ok(Self {
            tree,
            map: Some((map, arcs)),
            file,
        })
    }
}

type MySerializer<'a> = CompositeSerializer<
    AlignedSerializer<&'a mut AlignedVec>,
    FallbackScratch<HeapScratch<256>, AllocScratch>,
    SharedSerializeMap,
>;

impl<'a, K, V> RadixDb<'a, K, V> for InMemRadixDb<'a, K, V>
where
    K: TKey + Archive<Archived = K>,
    V: TValue + Archive<Archived = V>,
    K: for<'x> Serialize<MySerializer<'x>>,
    V: for<'x> Serialize<MySerializer<'x>>,
{
    fn tree(&self) -> &LazyRadixTree<'a, K, V> {
        &self.tree
    }

    fn tree_mut(&mut self) -> &mut LazyRadixTree<'a, K, V> {
        &mut self.tree
    }

    fn flush(&mut self) -> anyhow::Result<()>
    {
        let (map, mut arcs) = self.map.take().unwrap_or_default();
        println!("before {:?}", map);
        let mut serializer = CompositeSerializer::new(
            AlignedSerializer::new(&mut self.file),
            Default::default(),
            map,
        );
        serializer
            .serialize_value(&self.tree)
            .map_err(|e| anyhow::anyhow!("Error while serializing: {}", e))?;
        self.tree.all_arcs(&mut arcs);
        let (_, _, map) = serializer.into_components();
        println!("after {:?}", map);
        self.map = Some((map, arcs));
        Ok(())
    }
}

fn main() -> anyhow::Result<()> {
    let mut db: InMemRadixDb<u8, ()> = InMemRadixDb::default();
    for i in 0..100 {
        for j in 0..100 {
            let key = format!("{}-{}", i, j);
            db.tree_mut().union_with(&LazyRadixTree::single(key.as_bytes(), ()));
        }
        // db.flush()?;
        println!("{} {}", i, db.file.len());
    }
    db.flush()?;
    println!("{}", db.file.len());
    println!("db");
    for (k, v) in db.tree().iter() {
        println!("{}", std::str::from_utf8(&k)?);
    }
    let db2 = InMemRadixDb::<u8, ()>::load(&db.file)?;
    println!("db2");
    for (k, v) in db2.tree().iter() {
        println!("{}", std::str::from_utf8(&k)?);
    }

    println!("{} {}", db.file.len(), db2.file.len());
    Ok(())
}
