use super::*;

#[test]
fn interval_or_test() {
    // (25, 60) (-90, 25]
    let a = IntervalSeq::from_interval(&Interval::range(25, false, 60, false).unwrap());
    let b = IntervalSeq::from_interval(&Interval::range(-90, false, 25, true).unwrap());
    let c = a | b;
    println!("{}", c);
    assert!(c.is_valid())
}

#[test]
fn interval_xor_test() {
    // [-30, 11] [-98, -5)
    let a = IntervalSeq::from_interval(&Interval::range(-30, true, 11, true).unwrap());
    let b = IntervalSeq::from_interval(&Interval::range(-98, true, -5, false).unwrap());
    let c = a.clone() ^ b.clone();
    println!("a {:?}", a);
    println!("b {:?}", b);
    println!("c {:?}", c);
    assert!(c.is_valid())
}

#[test]
fn parse_tests() {
    assert_eq!("[0]".parse::<Interval<i32>>().unwrap(), Interval::at(0));
    assert_eq!(
        "(0,1)".parse::<Interval<i32>>().unwrap(),
        Interval::range(0, false, 1, false).unwrap()
    );
}
