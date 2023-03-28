<script>
  import { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate";

  import { LocalnetInfo } from "$lib/chain.local";

  import {
    offlineSigner as OfflineSignerStore,
    signingClient as SigningClientStore,
  } from "$lib/client.store.js";

  let promise = null;

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

    const offlineSigner = window.getOfflineSigner(LocalnetInfo.chainId);

    const accounts = await offlineSigner.getAccounts();

    const signingClient = new SigningCosmWasmClient(
      LocalnetInfo.rpc,
      accounts[0].address,
      offlineSigner
    );

    OfflineSignerStore.set(offlineSigner);
    SigningClientStore.set(signingClient);
  
    return accounts[0].address;
  }

  function connect() {
    promise = connectAsync();
  }
</script>

<div>
  {#await promise}
    <p>Connecting...</p>
  {:then address}
    {#if address}
      <p class="address">Connected: {address}</p>
    {:else}
      <button on:click={connect}> Connect </button>
    {/if}
  {:catch error}
    <p class="error">{error.message}</p>
    <button on:click={connect}> Retry </button>
  {/await}
</div>

<style>
  .error {
    color: #f00;
  }

  .address {
    color: #fff;
  }
</style>
