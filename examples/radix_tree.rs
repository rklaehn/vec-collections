use std::time::Instant;
use vec_collections::RadixTree;

fn main() {
    let t0 = Instant::now();
    let mut res = RadixTree::default();
    for i in 0..100000 {
        let key = i.to_string();
        let chars = key.chars().collect::<Vec<_>>();
        let node = RadixTree::single(&chars, i);
        res.union_with(&node);
    }
    println!("{}", t0.elapsed().as_secs_f64());
    // println!("{:#?}", res);
    let mut a: RadixTree<u8, i32> = RadixTree::single(b"aabbcc", 1);
    let b: RadixTree<u8, i32> = RadixTree::single(b"aabb", 2);
    let c: RadixTree<u8, i32> = RadixTree::single(b"aabbee", 3);
    println!("{:?}", a);
    a.union_with(&b);
    println!("{:?}", a);
}
