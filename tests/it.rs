use expect_test::Expect;
use serde::Serialize;

pub use expect_test::expect;

pub fn pretty<T: Serialize>(t: &T) -> String {
    ron::ser::to_string_pretty(
        t,
        ron::ser::PrettyConfig::default()
            .indentor("  ".to_owned())
            .separate_tuple_members(false),
    )
    .unwrap()
}

pub fn check(actual: impl ToString, expected: Expect) {
    expected.assert_eq(&actual.to_string())
}

pub mod referrals_core;

#[cfg(test)]
pub mod referrals_storage;

#[cfg(test)]
pub mod referrals_cw;

#[cfg(test)]
pub mod referrals_parse_cw;

#[cfg(test)]
pub mod referrals_archway_drivers;
