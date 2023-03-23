use expect_test::Expect;

pub use dbg_pls::pretty;
pub use expect_test::expect;

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
