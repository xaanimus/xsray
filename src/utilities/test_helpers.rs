/// for each element in the array, assert $cond.
/// cond is a condition given variables elem0 and elem1
/// that correspond to each pair of elements in
/// the two given arrays
macro_rules! assert_array_for_each {
    ($arr0: expr, $arr1: expr, $condfn: expr) => {
        let len0 = $arr0.len();
        let len1 = $arr1.len();

        if len0 != len1 {
            panic!(
                "{0}.len() != {1}.len(). {0}.len() == {2} and {1}.len() == {3}",
                stringify!($arr0), stringify!($arr1), len0, len1
            );
        }

        for i in 0..len0 {
            let (elem0, elem1) = ($arr0[i], $arr1[i]);
            let cond = $condfn(elem0, elem1);
            if !cond {
                panic!(
                    "with arguments {arr0}[{i}] and {arr1}[{i}], call to function ({condfn}) is false",
                    arr0 = stringify!($arr0),
                    arr1 = stringify!($arr1),
                    i = i,
                    condfn = stringify!($condfn)
                );
            }
        }
    }
}

macro_rules! assert_array_values_near_f32 {
    ($arr0: expr, $arr1: expr, $diff: expr) => {
        assert_array_for_each!($arr0, $arr1, |x: f32, y: f32| (x - y).abs() < ($diff))
    }
}

macro_rules! assert_array_eq {
    ($lhs: expr, $rhs: expr) => {
        let len0 = $lhs.len();
        let len1 = $rhs.len();

        if len0 != len1 {
            panic!(
                "{0} != {1}. {0}.len() == {2} and {1}.len() == {3}",
                stringify!($lhs), stringify!($rhs), len0, len1
            );
        }

        for i in 0..len0 {
            if $lhs[i] != $rhs[i] {
                panic!(
                    "{lhs} != {rhs}. value at index {i} differs",
                    lhs = stringify!($lhs),
                    rhs = stringify!($rhs),
                    i = i,
                );
            }
        }
    }
}

macro_rules! assert_near {
    ($lhs: expr, $rhs: expr, $diff: expr) => {
        let apprx_equal = ($lhs - $rhs).abs() < $diff;
        if !apprx_equal {
            panic!(
                "{lhs} is not apprx equal to {rhs}",
                lhs = stringify!($lhs),
                rhs = stringify!($rhs)
            )
        }
    }
}