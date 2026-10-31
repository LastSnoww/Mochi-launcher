<script lang="ts">
  import { accounts } from './lib/accounts.svelte';
  import AccountsView from './lib/components/AccountsView.svelte';
  import MainArea from './lib/components/MainArea.svelte';
  import Sidebar from './lib/components/Sidebar.svelte';
  import TitleBar from './lib/components/TitleBar.svelte';
  import { app } from './lib/state.svelte';

  accounts.init();

  $effect(() => {
    document.documentElement.toggleAttribute('data-paused', app.paused);
  });
</script>

<div class="flex h-screen flex-col">
  <TitleBar />
  <div class="flex min-h-0 flex-1">
    <Sidebar />
    {#if app.view === 'accounts'}
      <AccountsView />
    {:else}
      <MainArea />
    {/if}
  </div>
</div>
