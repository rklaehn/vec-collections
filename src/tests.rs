use super::*;
#[test]
fn parse_tests() {
    assert_eq!("[0]".parse::<Interval<i32>>().unwrap(), Interval::at(0));
    assert_eq!(
        "(0,1)".parse::<Interval<i32>>().unwrap(),
        Interval::range(0, false, 1, false).unwrap()
    );
}
