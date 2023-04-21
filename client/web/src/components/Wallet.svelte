<script>
  import { SigningCosmWasmClient, CosmWasmClient } from "@cosmjs/cosmwasm-stargate";

  import { LocalnetInfo } from "$lib/chain.local";

  import {
    offlineSigner as OfflineSignerStore,
    signingClient as SigningClientStore,
  } from "$lib/client.store.js";

  import { account } from "$lib/account.store.js";

  async function getAccountAsync(offlineSigner) {
    const accounts = await offlineSigner.getAccounts();

    const signingClient = new SigningCosmWasmClient(
      LocalnetInfo.rpc,
      accounts[0].address,
      offlineSigner
    );

    return [accounts[0], signingClient];
  }

  function onAccountChange() {
      console.log("account changed!");
      getAccountAsync($OfflineSignerStore).then((res) => {
        const [acc, client] = res;
        SigningClientStore.set(client);
        account.set(acc);
        console.log(account, "connected!");
      });
  }

  async function connectAsync() {
    if (!window.keplr) {
      throw new Error("Keplr not installed");
    }

    if (!window.keplr.experimentalSuggestChain) {
      throw new Error(
        "Experimental Keplr features required, please update Keplr."
      );
    }

    await window.keplr.experimentalSuggestChain(LocalnetInfo);

    await window.keplr.enable(LocalnetInfo.chainId);

    window.cosmwasmClient = CosmWasmClient;
    
    const offlineSigner = window.getOfflineSigner(LocalnetInfo.chainId);

    const [acc, signingClient] = await getAccountAsync(offlineSigner);

    OfflineSignerStore.set(offlineSigner);
    SigningClientStore.set(signingClient);
    account.set(acc);

    console.log(account, "connected!");

    window.addEventListener("keplr_keystorechange", onAccountChange);
  }

  function connect() {
    promise = connectAsync();
  }

  function disconnect() {
    window.removeEventListener("keplr_keystorechange", onAccountChange);
    OfflineSignerStore.set(null);
    SigningClientStore.set(null);
    account.set(null);
  }
</script>

<div>
  {#if $account}
    <p class="address">Connected: {$account.address}</p>
    <button on:click={disconnect}>Disconnect</button>
  {:else}
    <button on:click={connect}>Connect</button>
  {/if}
</div>

<style>
  .address {
    float: left;
    color: #fff;
  }

  button {
    float: right
  }
  
</style>
