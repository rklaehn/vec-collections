use super::*;

#[test]
fn intervalseq_parse_test() {
    let input =
        "[-98]; [-89]; [-83, -81); [-70, -48]; [-10, 23); [27, 76]; [80, 89]; [92, 93); (93, ∞)";
    input.parse::<IntervalSeq<i64>>().unwrap();
}

#[test]
fn intervalseq_or_test() {
    // (25, 60) (-90, 25]
    let a = IntervalSeq::from(Interval::range(25, false, 60, false));
    let b = IntervalSeq::from(Interval::range(-90, false, 25, true));
    let c = a | b;
    assert!(c.is_valid())
}

#[test]
fn interval_xor_test() {
    // [-30, 11] [-98, -5)
    let a = IntervalSeq::from(Interval::range(-30, true, 11, true));
    let b = IntervalSeq::from(Interval::range(-98, true, -5, false));
    let c = a.clone() ^ b.clone();
    assert!(c.is_valid())
}

#[test]
fn parse_tests() {
    assert_eq!("[0]".parse::<Interval<i32>>().unwrap(), Interval::at(0));
    assert_eq!(
        "(0,1)".parse::<Interval<i32>>().unwrap(),
        Interval::range(0, false, 1, false)
    );
}
