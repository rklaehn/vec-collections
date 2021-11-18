use rkyv::*;
use ser::Serializer;
use std::time::Instant;
use vec_collections::radix_tree::{
    AbstractRadixTree, AbstractRadixTreeMut, LazyRadixTree, RadixTree,
};

fn main() {
    let t0 = Instant::now();
    let mut eager = RadixTree::default();
    for i in 0..2 {
        let key = i.to_string();
        let chars = key.as_bytes().to_vec();
        let node = RadixTree::single(&chars, i);
        eager.union_with(&node);
    }
    println!("eager create {}", t0.elapsed().as_secs_f64());

    let mut serializer = ser::serializers::AllocSerializer::<256>::default();
    serializer.serialize_value(&eager).unwrap();
    let bytes = serializer.into_serializer().into_inner();
    println!(
        "hex dump of eager tree {:?}",
        eager.iter().collect::<Vec<_>>()
    );
    hexdump::hexdump(&bytes);

    let t0 = Instant::now();
    let mut lazy = LazyRadixTree::default();
    for i in 0..2 {
        let key = i.to_string();
        let chars = key.as_bytes().to_vec();
        let node = LazyRadixTree::single(&chars, i);
        lazy.union_with(&node);
    }
    println!("lazy create {}", t0.elapsed().as_secs_f64());

    let mut serializer = ser::serializers::AllocSerializer::<256>::default();
    serializer.serialize_value(&lazy).unwrap();
    let bytes = serializer.into_serializer().into_inner();
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
    tree.insert(&"fnord".as_bytes().to_vec(), 1);
    let mut serializer = ser::serializers::AllocSerializer::<256>::default();
    serializer.serialize_value(&tree).unwrap();
    let bytes2 = serializer.into_serializer().into_inner();
    println!(
        "hex dump of modified tree {:?}",
        tree.iter().collect::<Vec<_>>()
    );
    hexdump::hexdump(&bytes2);

    // println!("{:#?}", res);
    let mut a: RadixTree<u8, i32> = RadixTree::single(b"aabbcc", 1);
    let b: RadixTree<u8, i32> = RadixTree::single(b"aabb", 2);
    let _c: RadixTree<u8, i32> = RadixTree::single(b"aabbee", 3);
    println!("{:?}", a);
    a.union_with(&b);
    println!("{:?}", a);
}
