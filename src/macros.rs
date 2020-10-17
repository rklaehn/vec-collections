#[macro_export]
macro_rules! vecset {
    ($($x:expr),*$(,)*) => ({
        let mut set = $crate::VecSet::default();
        $(set.insert($x);)*
        set
    });
}

#[macro_export]
macro_rules! vecmap {
    ($($key:expr => $value:expr,)+) => { vecmap!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let mut _map = $crate::VecMap::default();
            $(
                let _ = _map.insert($key, $value);
            )*
            _map
        }
    };
}

#[cfg(test)]
mod tests {
    use crate::{VecMap, VecSet};

    #[test]
    fn vecset_macro() {
        let from_macro: VecSet<[u32; 4]> = vecset! {1,2,3};
        let mut manual: VecSet<[u32; 4]> = VecSet::default();
        manual.insert(1);
        manual.insert(2);
        manual.insert(3);
        assert_eq!(from_macro, manual);
    }

    #[test]
    fn vecmap_macro() {
        let from_macro: VecMap<[(u32, u32); 4]> = vecmap! {1 => 2, 2 => 4,3 => 6};
        let mut manual: VecMap<[(u32, u32); 4]> = VecMap::default();
        manual.insert(1, 2);
        manual.insert(2, 4);
        manual.insert(3, 6);
        assert_eq!(from_macro, manual);
    }
}
