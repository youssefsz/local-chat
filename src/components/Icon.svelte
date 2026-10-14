<script lang="ts">
  let { name, small = false }: { name: string; small?: boolean } = $props();
</script>

<svg class:small class="icon" aria-hidden="true"
  ><use href={'#i-' + name} /></svg
>
