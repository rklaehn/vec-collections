use std::{collections::BTreeMap, sync::Arc};

use futures::{
    channel::mpsc::{UnboundedReceiver, UnboundedSender},
    future,
    stream::BoxStream,
    StreamExt,
};
use rkyv::{
    archived_root,
    ser::serializers::{AlignedSerializer, CompositeSerializer},
    ser::{
        serializers::{AllocScratch, FallbackScratch, HeapScratch, SharedSerializeMap},
        Serializer,
    },
    AlignedVec, Archive, Serialize,
};
use vec_collections::{
    AbstractRadixTree, AbstractRadixTreeMut, ArchivedLazyRadixTree, LazyRadixTree, RadixTree, TKey,
    TValue,
};

struct Batch<'a, K: TKey, V: TValue> {
    v0: LazyRadixTree<'a, K, V>,
    v1: LazyRadixTree<'a, K, V>,
}

impl<'a, K: TKey, V: TValue> Batch<'a, K, V> {
    pub fn added(&self) -> LazyRadixTree<'a, K, V> {
        let mut res = self.v1.clone();
        res.difference_with(&self.v0);
        res
    }
    pub fn removed(&self) -> LazyRadixTree<'a, K, V> {
        let mut res = self.v0.clone();
        res.difference_with(&self.v1);
        res
    }
}

trait RadixDb<'a, K: TKey, V: TValue> {
    fn tree(&self) -> &LazyRadixTree<'a, K, V>;
    fn tree_mut(&mut self) -> &mut LazyRadixTree<'a, K, V>;
    fn flush(&mut self) -> anyhow::Result<()>;
    fn watch(&mut self) -> futures::channel::mpsc::UnboundedReceiver<LazyRadixTree<'a, K, V>>;
    fn watch_prefix(&mut self, prefix: Vec<K>) -> BoxStream<'a, Batch<'a, K, V>> {
        let tree = self.tree().clone();
        self.watch()
            .scan(tree, move |prev, curr| {
                let v0 = prev.filter_prefix(&prefix);
                let v1 = curr.filter_prefix(&prefix);
                future::ready(Some(Batch { v0, v1 }))
            })
            .boxed()
    }
}

struct InMemRadixDb<'a, K: TKey, V: TValue> {
    file: AlignedVec,
    map: Option<(
        SharedSerializeMap,
        BTreeMap<usize, Arc<Vec<LazyRadixTree<'a, K, V>>>>,
    )>,
    tree: LazyRadixTree<'a, K, V>,
    watchers: Vec<UnboundedSender<LazyRadixTree<'a, K, V>>>,
}

impl<'a, K: TKey, V: TValue> Default for InMemRadixDb<'a, K, V> {
    fn default() -> Self {
        Self {
            file: Default::default(),
            map: Default::default(),
            tree: Default::default(),
            watchers: Default::default(),
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
        let tree: &'a ArchivedLazyRadixTree<K, V> =
            unsafe { archived_root::<LazyRadixTree<K, V>>(bytes) };
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
            watchers: Default::default(),
        })
    }

    fn notify(&mut self) {
        let tree = self.tree.clone();
        self.watchers
            .retain(|sender| sender.unbounded_send(tree.clone()).is_ok())
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

    fn flush(&mut self) -> anyhow::Result<()> {
        let (map, mut arcs) = self.map.take().unwrap_or_default();
        // println!("before {:?}", map);
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
        // println!("after {:?}", map);
        self.map = Some((map, arcs));
        self.notify();
        Ok(())
    }

    fn watch(&mut self) -> UnboundedReceiver<LazyRadixTree<'a, K, V>> {
        let (s, r) = futures::channel::mpsc::unbounded();
        self.watchers.push(s);
        r
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let mut db: InMemRadixDb<u8, ()> = InMemRadixDb::default();
    let mut stream = db.watch_prefix("9".as_bytes().to_vec());
    tokio::spawn(async move {
        while let Some(x) = stream.next().await {
            for (added, _) in x.added().iter() {
                let text = std::str::from_utf8(&added).unwrap();
                if text.starts_with("990") {
                    println!("KAPUT!");
                    std::process::exit(1);
                }
                println!("added {}", text);
            }
            for (removed, _) in x.removed().iter() {
                let text = std::str::from_utf8(&removed).unwrap();
                println!("removed {}", text);
            }
        }
    });
    for i in 0..100 {
        for j in 0..100 {
            let key = format!("{}-{}", i, j);
            db.tree_mut()
                .union_with(&LazyRadixTree::single(key.as_bytes(), ()));
        }
        db.flush()?;
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
