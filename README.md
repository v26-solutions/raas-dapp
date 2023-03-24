# Referrals-as-a-Service (RaaS) dApp

## Overview

Hook into a network-wide referral program.

### For Contract Developers

Register your dApp with the service to incentivise 'Referrers' (e.g. web apps, telegram bots) to interface with your contract.

You specify what percentage of contract premiums to share with any referrers. 

This will drive traffic to your service while also making it more resilient with multiple, independent front-end providers.

Simply add a `referral-code` field to the messages you wish to incentivise, and forward the code to the RaaS contract when you recieve one of those messages. 

### For UI/UX & Chat Bot Experts

Register for a referral code and check out which dApps have registered, along with their proposed fee split. 

Drive traffic to those dApps to earn those referral rewards.

### Also...

[This dapp][1] [dog][2] [foods][3] [itself][4]!


## Development

```
❯ : cargo x
Usage: xtask <COMMAND>

Commands:
  coverage  run test coverage
  test      run tests
  dist      compile contracts for distribution
  dev       watch source files and run tests on changes
  install   install used cargo plugins (if not using Nix)
  archway   archway deployment tasks
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version
```

Cargo will automatically build the `xtask` binary when you run `cargo x` for the first time.

### Archway

Currently a [patched version][5] of `archwayd` is required to get the following features:
- Allow contracts to create their own x/rewards metadata entries. [RFC #328][6]
- Allow contracts to change the metadata of other contracts (of which they are the x/rewards owner). [PR #326][7]

To deploy locally against this version, create a `.env` file in the repository root that looks like this:

```
❯ : cat .env  

ARCHWAY_REPO_URL = "https://github.com/chris-ricketts/archway"

ARCHWAY_REPO_BRANCH = "v26/next"
```


[1]: https://github.com/v26-solutions/raas-dapp/blob/ab6878c33fbe1de87c0e181df39f7bde717cd32d/crates/cosmwasm/archway/drivers/hub.rs#L164 
[2]: https://github.com/v26-solutions/raas-dapp/blob/ab6878c33fbe1de87c0e181df39f7bde717cd32d/tests/referrals_archway_drivers/hub.rs#L112
[3]: https://github.com/v26-solutions/raas-dapp/blob/ab6878c33fbe1de87c0e181df39f7bde717cd32d/crates/cosmwasm/archway/drivers/hub.rs#L192
[4]: https://github.com/v26-solutions/raas-dapp/blob/ab6878c33fbe1de87c0e181df39f7bde717cd32d/tests/referrals_archway_drivers/hub.rs#L352
[5]: https://github.com/chris-ricketts/archway/tree/v26/next
[6]: https://github.com/archway-network/archway/issues/328
[7]: https://github.com/archway-network/archway/pull/326
