<script lang="ts">
  import MainArea from './lib/components/MainArea.svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import TitleBar from './lib/components/TitleBar.svelte';
  import { app } from './lib/state.svelte';

  $effect(() => {
    document.documentElement.toggleAttribute('data-paused', app.paused);
  });
</script>

<div class="flex h-screen flex-col">
  <TitleBar />
  <div class="flex min-h-0 flex-1">
    <Sidebar />
    <MainArea />
  </div>
</div>
