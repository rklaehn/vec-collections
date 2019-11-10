use std::collections::BTreeSet;
use std::fmt::Debug;

///
/// A support trait for testing any kind of collection.
///
/// Almost anything can be viewed as a collection. E.g. an integer can be viewed as a collection of bit offsets at which it has
/// a boolean value.
///
pub trait TestSamples<K, V> {
    /// produces "interesting" sample points to test a property for.
    fn samples(&self, res: &mut BTreeSet<K>);

    /// gets the value of the collection at position k
    fn at(&self, k: K) -> V;
}

pub fn unary_element_test<C, K, V>(a: &C, r: C, op: impl Fn(V) -> V) -> bool
where
    C: TestSamples<K, V> + Debug + Clone,
    K: Ord + Clone + Debug,
    V: Eq + Debug,
{
    let mut s: BTreeSet<K> = BTreeSet::new();
    a.samples(&mut s);
    r.samples(&mut s);
    s.into_iter().all(|key| {
        let value = a.at(key.clone());
        let actual = op(value);
        let expected = r.at(key.clone());
        if expected != actual {
            println!(
                "expected!=actual at: {:?}. {:?}!={:?}",
                key, expected, actual
            );
            println!("a: {:?}", a);
            println!("r: {:?}", r);
            false
        } else {
            true
        }
    })
}

pub fn binary_element_test<C, K, V>(a: &C, b: &C, r: C, op: impl Fn(V, V) -> V) -> bool
where
    C: TestSamples<K, V> + Debug,
    K: Ord + Clone + Debug,
    V: Eq + Debug,
{
    let mut s: BTreeSet<K> = BTreeSet::new();
    a.samples(&mut s);
    b.samples(&mut s);
    r.samples(&mut s);
    s.into_iter().all(|key| {
        let a_value = a.at(key.clone());
        let b_value = b.at(key.clone());
        let actual = op(a_value, b_value);
        let expected = r.at(key.clone());
        if expected != actual {
            println!(
                "expected!=actual at: {:?}. {:?}!={:?}",
                key, expected, actual
            );
            println!("a: {:?}", a);
            println!("b: {:?}", b);
            println!("r: {:?}", r);
            false
        } else {
            true
        }
    })
}

pub fn binary_property_test<C, K, V>(a: &C, b: &C, r: bool, op: impl Fn(V, V) -> bool) -> bool
where
    C: TestSamples<K, V> + Debug,
    K: Ord + Clone + Debug,
    V: Eq + Debug,
{
    let mut s: BTreeSet<K> = BTreeSet::new();
    a.samples(&mut s);
    b.samples(&mut s);
    if r {
        s.iter().cloned().all(|e| op(a.at(e.clone()), b.at(e)))
    } else {
        s.iter().cloned().any(|e| !op(a.at(e.clone()), b.at(e)))
    }
}
