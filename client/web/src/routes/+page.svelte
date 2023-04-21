<script>
  import { dapps } from '$lib/dapps.store.js';
  
  import Section from "../components/Section.svelte";

  dapps.get().then(res => console.log(res));

</script>

<Section title="REFERRAL REWARDS">
  <p>TODO</p>
</Section>

<Section title="DAPPS">
  <p>TODO</p>
</Section>
