use std::time::Instant;
use vec_collections::{AbstractRadixTree, AbstractRadixTreeMut, LazyRadixTree, RadixTree};

fn main() {
    let t0 = Instant::now();
    let mut res = LazyRadixTree::default();
    for i in 0..100000 {
        let key = i.to_string();
        let chars = key.chars().collect::<Vec<_>>();
        let node = LazyRadixTree::single(&chars, i);
        res.union_with(&node);
    }
    println!("eager create {}", t0.elapsed().as_secs_f64());

    let t0 = Instant::now();
    let mut res = RadixTree::default();
    for i in 0..100000 {
        let key = i.to_string();
        let chars = key.chars().collect::<Vec<_>>();
        let node = RadixTree::single(&chars, i);
        res.union_with(&node);
    }
    println!("lazy create {}", t0.elapsed().as_secs_f64());

    use rkyv::*;
    use ser::Serializer;
    let mut serializer = ser::serializers::AllocSerializer::<256>::default();
    serializer.serialize_value(&res).unwrap();
    let bytes = serializer.into_serializer().into_inner();
    let archived = unsafe { rkyv::archived_root::<RadixTree<char, i32>>(&bytes) };
    let mut tree = LazyRadixTree::from(archived);
    for (k, v) in tree.iter() {
        println!("{:?} {}", k, v);
    }
    tree.union_with(&LazyRadixTree::single(
        &"fnord".chars().collect::<Vec<_>>(),
        1,
    ));
    for (k, v) in tree.iter() {
        println!("{:?} {}", k, v);
    }

    // println!("{:#?}", res);
    let mut a: RadixTree<u8, i32> = RadixTree::single(b"aabbcc", 1);
    let b: RadixTree<u8, i32> = RadixTree::single(b"aabb", 2);
    let c: RadixTree<u8, i32> = RadixTree::single(b"aabbee", 3);
    println!("{:?}", a);
    a.union_with(&b);
    println!("{:?}", a);
}
