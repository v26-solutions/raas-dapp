use std::collections::BTreeMap;

use kv_storage::{Deserializer, Fallible, HasKey, KvStore, Read, Remove, Serializer, Write};
use serde::{de::DeserializeOwned, Serialize};

use referrals_core::hub::{
    MutableCollectStore, MutableDappStore, MutableReferralStore, ReadonlyCollectStore,
    ReadonlyDappStore, ReadonlyReferralStore, ReferralCode,
};
use referrals_core::Id;
use referrals_storage::Storage as CoreStorage;

use crate::{check, expect, nz, nzp};

#[derive(Default)]
pub struct RonSerde(String);

#[derive(Default)]
pub struct Repo(BTreeMap<String, String>);

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    Ser(#[from] ron::Error),
    #[error(transparent)]
    De(#[from] ron::error::SpannedError),
}

impl Fallible for RonSerde {
    type Error = Error;
}

impl Serializer for RonSerde {
    fn serialize<T: Serialize>(&mut self, item: &T) -> Result<&[u8], Self::Error> {
        self.0 = ron::to_string(item)?;
        Ok(self.0.as_bytes())
    }
}

impl Deserializer for RonSerde {
    fn deserialize<T: DeserializeOwned>(bytes: Vec<u8>) -> Result<T, Self::Error> {
        ron::de::from_bytes(&bytes).map_err(Error::from)
    }
}

impl Fallible for Repo {
    type Error = std::convert::Infallible;
}

impl Read for Repo {
    fn read(&self, key: &[u8]) -> Result<Option<Vec<u8>>, Self::Error> {
        Ok(self
            .0
            .get(std::str::from_utf8(key).unwrap())
            .cloned()
            .map(String::into_bytes))
    }
}

impl Write for Repo {
    fn write(&mut self, key: &[u8], bytes: &[u8]) -> Result<(), Self::Error> {
        self.0.insert(
            String::from_utf8(key.to_owned()).unwrap(),
            String::from_utf8(bytes.to_owned()).unwrap(),
        );
        Ok(())
    }
}

impl HasKey for Repo {
    fn has_key(&self, key: &[u8]) -> Result<bool, Self::Error> {
        Ok(self.0.contains_key(std::str::from_utf8(key).unwrap()))
    }
}

impl Remove for Repo {
    fn remove(&mut self, key: &[u8]) -> Result<(), Self::Error> {
        self.0.remove(std::str::from_utf8(key).unwrap());
        Ok(())
    }
}

#[test]
fn dapp_storage_works() {
    let mut storage: CoreStorage<KvStore<RonSerde, Repo>> = CoreStorage::new(KvStore::default());

    let id1 = Id::from("id1");
    let id2 = Id::from("id2");

    storage.add_dapp(&id1, "dapp1".to_owned()).unwrap();
    storage.set_percent(&id1, nzp!(100)).unwrap();
    storage.set_collector(&id1, Id::from("collector")).unwrap();
    storage.set_repo_url(&id1, "some_repo".to_owned()).unwrap();
    storage
        .set_rewards_pot(&id1, Id::from("rewards_pot_1"))
        .unwrap();
    storage.add_dapp(&id2, "dapp2".to_owned()).unwrap();
    storage.set_percent(&id2, nzp!(75)).unwrap();
    storage
        .set_collector(&id2, Id::from("another_collector"))
        .unwrap();
    storage
        .set_repo_url(&id2, "some_other_repo".to_owned())
        .unwrap();
    storage
        .set_rewards_pot(&id2, Id::from("rewards_pot_2"))
        .unwrap();

    check(
        storage.inner().repo(),
        expect![[r#"
            {
            	referrals_storage::hub::dapp::collector::id1 => "collector"
            	referrals_storage::hub::dapp::collector::id2 => "another_collector"
            	referrals_storage::hub::dapp::dapp_index::00000000 => "id1"
            	referrals_storage::hub::dapp::dapp_index::00000001 => "id2"
            	referrals_storage::hub::dapp::dapp_last_index => 1
            	referrals_storage::hub::dapp::dapp_reverse_index::id1 => 0
            	referrals_storage::hub::dapp::dapp_reverse_index::id2 => 1
            	referrals_storage::hub::dapp::dapps::id1 => "dapp1"
            	referrals_storage::hub::dapp::dapps::id2 => "dapp2"
            	referrals_storage::hub::dapp::percent::id1 => 100
            	referrals_storage::hub::dapp::percent::id2 => 75
            	referrals_storage::hub::dapp::repo_url::id1 => "some_repo"
            	referrals_storage::hub::dapp::repo_url::id2 => "some_other_repo"
            	referrals_storage::hub::dapp::rewards_pot::id1 => "rewards_pot_1"
            	referrals_storage::hub::dapp::rewards_pot::id2 => "rewards_pot_2"
            }
        "#]],
    );

    assert!(storage.dapp_exists(&id1).unwrap());

    storage.remove_dapp(&id1).unwrap();

    assert!(!storage.dapp_exists(&id1).unwrap());

    assert!(storage.dapp_exists(&id2).unwrap());

    check(storage.percent(&id2).unwrap().to_u8(), expect!["75"]);

    check(
        storage.collector(&id2).unwrap().into_string(),
        expect!["another_collector"],
    );

    check(storage.has_rewards_pot(&id2).unwrap(), expect!["true"]);

    check(
        storage.rewards_pot(&id2).unwrap().into_string(),
        expect!["rewards_pot_2"],
    );

    storage
        .add_dapp(&Id::from("no_pot"), "no_pot".to_owned())
        .unwrap();

    assert!(!storage.has_rewards_pot(&Id::from("no_pot")).unwrap());
}

#[test]
fn referral_storage_works() {
    let mut storage: CoreStorage<KvStore<RonSerde, Repo>> = CoreStorage::new(KvStore::default());

    let code1 = ReferralCode::from(1);
    let code2 = ReferralCode::from(2);
    let code3 = ReferralCode::from(3);
    let id1 = Id::from("id1");
    let id2 = Id::from("id2");
    let id3 = Id::from("id3");
    let dapp1 = Id::from("dapp1");
    let dapp2 = Id::from("dapp2");
    let dapp3 = Id::from("dapp3");

    storage.set_code_owner(code1, id1.clone()).unwrap();

    storage.set_latest(code2).unwrap();

    storage.set_code_owner(code2, id2.clone()).unwrap();

    storage.increment_invocations(&dapp1, code1).unwrap();

    storage.increment_invocations(&dapp2, code1).unwrap();

    storage.increment_invocations(&dapp2, code2).unwrap();

    storage.set_total_earnings(code1, nz!(2000)).unwrap();

    storage.set_total_earnings(code2, nz!(1000)).unwrap();

    storage.set_dapp_earnings(&dapp1, code1, nz!(1000)).unwrap();

    storage.set_dapp_earnings(&dapp2, code1, nz!(1000)).unwrap();

    storage.set_dapp_earnings(&dapp2, code2, nz!(1000)).unwrap();

    storage.set_dapp_contributions(&dapp1, nz!(1000)).unwrap();

    storage.set_dapp_contributions(&dapp2, nz!(2000)).unwrap();

    check(
        storage.inner().repo(),
        expect![[r#"
            {
            	referrals_storage::hub::referral::code_dapp_earnings::dapp1:00000001 => 1000
            	referrals_storage::hub::referral::code_dapp_earnings::dapp2:00000001 => 1000
            	referrals_storage::hub::referral::code_dapp_earnings::dapp2:00000002 => 1000
            	referrals_storage::hub::referral::code_owners::id1 => 1
            	referrals_storage::hub::referral::code_owners::id2 => 2
            	referrals_storage::hub::referral::code_total_earnings::00000001 => 2000
            	referrals_storage::hub::referral::code_total_earnings::00000002 => 1000
            	referrals_storage::hub::referral::codes::00000001 => "id1"
            	referrals_storage::hub::referral::codes::00000002 => "id2"
            	referrals_storage::hub::referral::dapp_contributions::dapp1 => 1000
            	referrals_storage::hub::referral::dapp_contributions::dapp2 => 2000
            	referrals_storage::hub::referral::discrete_referrers::dapp1 => 1
            	referrals_storage::hub::referral::discrete_referrers::dapp2 => 2
            	referrals_storage::hub::referral::invocation_counts::dapp1:00000001 => 1
            	referrals_storage::hub::referral::invocation_counts::dapp2:00000001 => 1
            	referrals_storage::hub::referral::invocation_counts::dapp2:00000002 => 1
            	referrals_storage::hub::referral::latest_code => 2
            	referrals_storage::hub::referral::total_invocation_counts::dapp1 => 1
            	referrals_storage::hub::referral::total_invocation_counts::dapp2 => 2
            }
        "#]],
    );

    assert!(storage.code_exists(code1).unwrap());
    assert!(storage.code_exists(code2).unwrap());
    assert!(!storage.code_exists(code3).unwrap());

    assert!(storage.owner_exists(&id1).unwrap());
    assert!(storage.owner_exists(&id2).unwrap());
    assert!(!storage.owner_exists(&id3).unwrap());

    check(
        storage.owner_of(code1).unwrap().unwrap().into_string(),
        expect!["id1"],
    );

    check(
        storage.owner_of(code2).unwrap().unwrap().into_string(),
        expect!["id2"],
    );

    assert!(storage.owner_of(code3).unwrap().is_none());

    check(storage.latest().unwrap().unwrap().to_u64(), expect!["2"]);

    check(
        storage.total_earnings(code1).unwrap().unwrap(),
        expect!["2000"],
    );

    check(
        storage.total_earnings(code2).unwrap().unwrap(),
        expect!["1000"],
    );

    assert!(storage.total_earnings(code3).unwrap().is_none());

    check(
        storage.dapp_earnings(&dapp1, code1).unwrap().unwrap(),
        expect!["1000"],
    );

    check(
        storage.dapp_earnings(&dapp2, code2).unwrap().unwrap(),
        expect!["1000"],
    );

    assert!(storage.dapp_earnings(&dapp1, code2).unwrap().is_none());

    check(
        storage.dapp_contributions(&dapp1).unwrap().unwrap(),
        expect!["1000"],
    );

    check(
        storage.dapp_contributions(&dapp2).unwrap().unwrap(),
        expect!["2000"],
    );

    assert!(storage.dapp_contributions(&dapp3).unwrap().is_none());
}

#[test]
fn collect_storage_works() {
    let mut storage: CoreStorage<KvStore<RonSerde, Repo>> = CoreStorage::new(KvStore::default());

    let code1 = ReferralCode::from(1);
    let code2 = ReferralCode::from(2);
    let code3 = ReferralCode::from(3);
    let dapp1 = Id::from("dapp1");
    let dapp2 = Id::from("dapp2");
    let dapp3 = Id::from("dapp3");

    storage
        .set_referrer_total_collected(code1, nz!(1000))
        .unwrap();
    storage
        .set_referrer_dapp_collected(&dapp1, code1, nz!(500))
        .unwrap();
    storage
        .set_referrer_dapp_collected(&dapp2, code1, nz!(500))
        .unwrap();

    storage
        .set_referrer_total_collected(code2, nz!(3000))
        .unwrap();
    storage
        .set_referrer_dapp_collected(&dapp1, code2, nz!(2000))
        .unwrap();
    storage
        .set_referrer_dapp_collected(&dapp2, code2, nz!(1000))
        .unwrap();

    storage.set_dapp_total_collected(&dapp1, nz!(200)).unwrap();
    storage.set_dapp_total_collected(&dapp2, nz!(500)).unwrap();

    check(
        storage.inner().repo(),
        expect![[r#"
            {
            	referrals_storage::hub::collect::dapp_total::dapp1 => 200
            	referrals_storage::hub::collect::dapp_total::dapp2 => 500
            	referrals_storage::hub::collect::referrer_dapp::dapp1:00000001 => 500
            	referrals_storage::hub::collect::referrer_dapp::dapp1:00000002 => 2000
            	referrals_storage::hub::collect::referrer_dapp::dapp2:00000001 => 500
            	referrals_storage::hub::collect::referrer_dapp::dapp2:00000002 => 1000
            	referrals_storage::hub::collect::referrer_total::00000001 => 1000
            	referrals_storage::hub::collect::referrer_total::00000002 => 3000
            }
        "#]],
    );

    check(
        storage.referrer_total_collected(code1).unwrap().unwrap(),
        expect!["1000"],
    );

    check(
        storage.referrer_total_collected(code2).unwrap().unwrap(),
        expect!["3000"],
    );

    assert!(storage.referrer_total_collected(code3).unwrap().is_none());

    check(
        storage
            .referrer_dapp_collected(&dapp1, code1)
            .unwrap()
            .unwrap(),
        expect!["500"],
    );

    check(
        storage
            .referrer_dapp_collected(&dapp1, code2)
            .unwrap()
            .unwrap(),
        expect!["2000"],
    );

    check(
        storage
            .referrer_dapp_collected(&dapp2, code1)
            .unwrap()
            .unwrap(),
        expect!["500"],
    );

    check(
        storage
            .referrer_dapp_collected(&dapp2, code2)
            .unwrap()
            .unwrap(),
        expect!["1000"],
    );

    assert!(storage
        .referrer_dapp_collected(&dapp1, code3)
        .unwrap()
        .is_none());

    assert!(storage
        .referrer_dapp_collected(&dapp2, code3)
        .unwrap()
        .is_none());

    check(
        storage.dapp_total_collected(&dapp1).unwrap().unwrap(),
        expect!["200"],
    );

    check(
        storage.dapp_total_collected(&dapp2).unwrap().unwrap(),
        expect!["500"],
    );

    assert!(storage.dapp_total_collected(&dapp3).unwrap().is_none());
}

impl std::fmt::Display for Repo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "{{")?;

        for (key, value) in self.0.iter() {
            let key: String = key
                .chars()
                .map(|c| {
                    if c.is_control() {
                        char::from_digit(c.into(), 10).unwrap()
                    } else {
                        c
                    }
                })
                .collect();
            writeln!(f, "\t{key} => {value}")?;
        }

        writeln!(f, "}}")
    }
}
