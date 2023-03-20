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

## Development

```
❯ : cargo x
Usage: xtask <COMMAND>

Commands:
  coverage  run test coverage
  test      run tests
  dist      compile contracts for distribution
  dev       watch source files and re-run 'xtask test --update --backtrace' when saving files
  install   install used cargo plugins (if not using Nix)
  help      Print this message or the help of the given subcommand(s)

Options:
  -h, --help     Print help
  -V, --version  Print version 
```

Cargo will automatically build the `xtask` binary when you run this for the first time.

