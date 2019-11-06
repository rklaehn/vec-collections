/// checks that bitops are symmetric
macro_rules! bitop_symmetry {
    ($test:ty) => {
        #[quickcheck]
        fn bitor_symmetric(a: $test, b: $test) -> bool {
            &a | &b == &b | &a
        }

        #[quickcheck]
        fn bitxor_symmetric(a: $test, b: $test) -> bool {
            &a ^ &b == &b ^ &a
        }

        #[quickcheck]
        fn bitand_symmetric(a: $test, b: $test) -> bool {
            &a & &b == &b & &a
        }
    };
}

/// checks properties of the empty element vs. bitops and sub
macro_rules! bitop_empty {
    ($test:ty) => {
        #[quickcheck]
        fn bitand_empty_neutral(a: $test) -> bool {
            let e = Test::empty();
            (&a & &e) == e && (&e & &a) == e
        }

        #[quickcheck]
        fn bitor_empty_neutral(a: $test) -> bool {
            let e = Test::empty();
            (&a | &e) == a && (&e | &a) == a
        }

        #[quickcheck]
        fn bitxor_empty_neutral(a: $test) -> bool {
            let e = Test::empty();
            (&a ^ &e) == a && (&e ^ &a) == a
        }

        #[quickcheck]
        fn sub_empty_neutral(a: $test) -> bool {
            let e = Test::empty();
            (&a - &e) == a
        }
    };
}

/// checks properties of the all element vs. bitops and sub/not
macro_rules! bitop_sub_not_all {
    ($test:ty) => {
        #[quickcheck]
        fn bitand_all_neutral(x: $test) -> bool {
            let a = Test::all();
            (&x & &a) == x && (&a & &x) == x
        }

        #[quickcheck]
        fn bitor_all_all(x: $test) -> bool {
            let a = Test::all();
            (&x | &a) == a && (&a | &x) == a
        }

        #[quickcheck]
        fn bitxor_all_not(x: $test) -> bool {
            let a = Test::all();
            (&a ^ &x) == !&x && (&x ^ &a) == !&x
        }

        #[quickcheck]
        fn sub_all_not(x: $test) -> bool {
            let a = Test::all();
            (&a - &x) == !&x
        }

        #[quickcheck]
        fn sub_bitand_not(a: $test, b: $test) -> bool {
            a.sub(&b) == a.bitand(&b.not())
        }

        #[quickcheck]
        fn not_not_neutral(a: $test) -> bool {
            &a == &((&a).not()).not()
        }

        #[quickcheck]
        fn all_is_all(a: $test) -> bool {
            let r1 = a.is_all();
            let r2 = a == Test::all();
            r1 == r2
        }
    };
}

/// checks consistency between op and op_assign
macro_rules! bitop_assign_consistent {
    ($test:ty) => {
        #[quickcheck]
        fn bitand_assign_consistent(a: $test, b: $test) -> bool {
            let r1 = &a & &b;
            let mut r2 = a;
            r2 &= b;
            r1 == r2
        }
        #[quickcheck]
        fn bitor_assign_consistent(a: $test, b: $test) -> bool {
            let r1 = &a | &b;
            let mut r2 = a;
            r2 |= b;
            r1 == r2
        }
        #[quickcheck]
        fn bitxor_assign_consistent(a: $test, b: $test) -> bool {
            let r1 = &a ^ &b;
            let mut r2 = a;
            r2 ^= b;
            r1 == r2
        }
        #[quickcheck]
        fn sub_assign_consistent(a: $test, b: $test) -> bool {
            let r1 = &a - &b;
            let mut r2 = a;
            r2 -= b;
            r1 == r2
        }
    };
}

// checks that the set predicates is_disjoint, is_subset, is_empty, is_all etc. are consistent with the operations
macro_rules! set_predicate_consistent {
    ($test:ty) => {
        #[quickcheck]
        fn is_disjoint_bitand_consistent(a: $test, b: $test) -> bool {
            let r1 = a.is_disjoint(&b);
            let r2 = (&a & &b).is_empty();
            r1 == r2
        }

        #[quickcheck]
        fn is_subset_bitand_consistent(a: $test, b: $test) -> bool {
            let r1 = a.is_subset(&b);
            let r2 = &(&a & &b) == &a;
            r1 == r2
        }

        #[quickcheck]
        fn is_subset_is_superset_consistent(a: $test, b: $test) -> bool {
            let r1 = a.is_subset(&b);
            let r2 = b.is_superset(&a);
            r1 == r2
        }

        #[quickcheck]
        fn empty_is_empty_consistent(a: $test) -> bool {
            let r1 = a.is_empty();
            let r2 = a == Test::empty();
            r1 == r2
        }
    };
}
