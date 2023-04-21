import { writable } from 'svelte/store';

import { asyncable } from 'svelte-asyncable';

import { CosmWasmClient } from "@cosmjs/cosmwasm-stargate";

import { LocalnetInfo } from "$lib/chain.local";

export const offlineSigner = writable(null);

export const signingClient = writable(null);

export const queryClient = asyncable(async () => {
  CosmWasmClient.connect(LocalnetInfo.rpc)
});