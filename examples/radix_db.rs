use rkyv::{AlignedVec, Archive, Deserialize, Serialize, archived_root, de::deserializers::SharedDeserializeMap, ser::serializers::{AlignedSerializer, CompositeSerializer, FallbackScratch}, ser::{
        serializers::{AllocSerializer, HeapScratch, SharedSerializeMap},
        Serializer,
    }};
use std::{io::Read, time::Instant};
use vec_collections::{AbstractRadixTree, AbstractRadixTreeMut, ArchivedRadixTree2, LazyRadixTree, TKey, TValue};

trait RadixDb<'a, K: TKey, V: TValue> {
    fn tree(&self) -> &LazyRadixTree<'a, K, V>;
    fn tree_mut(&mut self) -> &mut LazyRadixTree<'a, K, V>;
    fn flush(&mut self) -> anyhow::Result<()>;
}

struct InMemRadixDb<'a, K: TKey, V: TValue> {
    file: AlignedVec,
    map: Option<SharedSerializeMap>,
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
        // this is a lie
        let bytes: &'a [u8] = unsafe { std::mem::transmute(bytes) };
        let tree: &'a ArchivedRadixTree2<K, V> = unsafe { archived_root::<LazyRadixTree<K, V>>(bytes) };
        let tree: LazyRadixTree<'a, K, V> = LazyRadixTree::from(tree);
        let mut file = AlignedVec::new();
        let mut serializer = CompositeSerializer::new(
            AlignedSerializer::new(&mut file),
            HeapScratch::default(),
            SharedSerializeMap::default(),
        );
        // this makes the lie true
        serializer
            .serialize_value(&tree)
            .map_err(|e| anyhow::anyhow!("Error while serializing: {}", e))?;
        let (_, _, map) = serializer.into_components();
        Ok(Self {
            tree,
            map: Some(map),
            file,
        })
    }
}

type MySerializer<'a> = CompositeSerializer<
    AlignedSerializer<&'a mut AlignedVec>,
    HeapScratch<256>,
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
        let map = self.map.take().unwrap_or_default();
        let mut serializer = CompositeSerializer::new(
            AlignedSerializer::new(&mut self.file),
            HeapScratch::default(),
            map,
        );
        serializer
            .serialize_value(&self.tree)
            .map_err(|e| anyhow::anyhow!("Error while serializing: {}", e))?;
        let (_, _, map) = serializer.into_components();
        self.map = Some(map);
        Ok(())
    }
}

fn main() {
    let t0 = Instant::now();
    let mut lazy = LazyRadixTree::default();
    for i in 0..2 {
        let key = i.to_string();
        let chars = key.as_bytes().to_vec();
        let node = LazyRadixTree::single(&chars, i);
        lazy.union_with(&node);
    }
    println!("lazy create {}", t0.elapsed().as_secs_f64());

    let mut serializer = rkyv::ser::serializers::AllocSerializer::<256>::default();
    serializer.serialize_value(&lazy).unwrap();
    let (serializer, scratch, map) = serializer.into_components();
    let bytes = serializer.into_inner();
    println!(
        "hex dump of lazy tree {:?}",
        lazy.iter().collect::<Vec<_>>()
    );
    hexdump::hexdump(&bytes);

    let archived = unsafe { rkyv::archived_root::<LazyRadixTree<u8, i32>>(&bytes) };
    let mut tree = LazyRadixTree::from(archived);
    for (k, v) in tree.iter() {
        println!("{:?} {}", k, v);
    }
    tree.union_with(&LazyRadixTree::single(&"fnord".as_bytes().to_vec(), 1));
    let mut serializer = rkyv::ser::serializers::AllocSerializer::<256>::default();
    serializer.serialize_value(&tree).unwrap();
    let bytes2 = serializer.into_serializer().into_inner();
    println!(
        "hex dump of modified tree {:?}",
        tree.iter().collect::<Vec<_>>()
    );
    hexdump::hexdump(&bytes2);
}
