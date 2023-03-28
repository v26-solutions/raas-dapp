<script>
  export let title;
</script>

<fieldset>
  <legend>{title}</legend>

  <slot />
</fieldset>

<style>
  fieldset {
    margin-top: 12px;
    border: 1px solid #39c;
    padding: 12px;
    border-radius: 8px;
  }
</style>
