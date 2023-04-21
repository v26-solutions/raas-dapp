import { asyncable } from 'svelte-asyncable';

import { queryClient } from "$lib/client.store";

import {
  PUBLIC_REFERRAL_HUB_ADDRESS as hubAddr,
} from "$env/static/public";

export const dapps = asyncable(
  // async (client) => await client.queryContractSmart(hubAddr, { "all_dapps": {} }),
  async (connectedClient) => {
    let client = connectedClient;
    console.log(client)
  },
  undefined,
  [ queryClient ]
);