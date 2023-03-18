use expect_test::Expect;

pub use dbg_pls::pretty;
pub use expect_test::expect;

pub fn debug<T: std::fmt::Debug>(t: T) -> String {
    format!("{t:?}")
}

pub fn debug_slice<T: std::fmt::Debug>(slice: &[T]) -> String {
    use std::fmt::Write;

    let mut s = String::new();

    writeln!(&mut s, "[").unwrap();

    for item in slice {
        writeln!(&mut s, "\t{item:?}").unwrap();
    }

    writeln!(&mut s, "]").unwrap();

    s
}

pub fn check(actual: impl ToString, expected: Expect) {
    expected.assert_eq(&actual.to_string())
}

pub mod referrals_core;
