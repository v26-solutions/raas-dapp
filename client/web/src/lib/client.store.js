import { writable, readable } from 'svelte/store';

import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate";

import { LocalnetInfo } from "$lib/chain.local";

export const offlineSigner = writable(null);

export const signingClient = writable(null);

export const queryClient = readable(null, set => {
  set(new CosmWasmClient(LocalnetInfo.rpc))
});