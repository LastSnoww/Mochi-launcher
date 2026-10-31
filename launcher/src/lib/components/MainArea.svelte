<script lang="ts">
  import { FolderOpen, Package, Play, ScrollText, SlidersHorizontal, Square } from '@lucide/svelte';
  import ModeIcon from '../icons/ModeIcon.svelte';
  import { loaderNames } from '../mock';
  import { app } from '../state.svelte';

  const actions = [
    { label: 'Mods', icon: Package },
    { label: 'Settings', icon: SlidersHorizontal },
    { label: 'Open folder', icon: FolderOpen },
    { label: 'Logs', icon: ScrollText },
  ];
</script>

<main class="flex min-w-0 flex-1 flex-col gap-2 bg-sage p-6">
  {#if app.selected}
    {@const instance = app.selected}
    <p class="text-xs font-bold text-sage-ink-muted">Selected instance</p>
    <section class="flex items-center gap-5 rounded-card bg-sage-deep p-5">
      <div class="grid size-18 shrink-0 place-items-center rounded-card bg-blush">
        <ModeIcon {instance} size={40} />
      </div>
      <div class="min-w-0 flex-1">
        <h1 class="truncate text-2xl font-bold text-sage-ink">{instance.name}</h1>
        <p class="text-sage-ink">
          Minecraft {instance.mcVersion}{instance.loader ? ` · ${loaderNames[instance.loader]}` : ' · Vanilla'}
        </p>
        <div class="mt-3 flex flex-wrap gap-2">
          {#if instance.type !== 'vanilla'}
            <span class="rounded-chip bg-sage px-2 py-0.5 text-xs font-bold text-sage-ink-muted">
              {instance.modCount} mods
            </span>
          {/if}
          {#if instance.type === 'pvp'}
            <span class="rounded-chip bg-sage px-2 py-0.5 text-xs font-bold text-sage-ink-muted">
              Client mod {instance.clientMod ? 'on' : 'off'}
            </span>
          {/if}
        </div>
      </div>
    </section>

    <p class="mt-4 text-xs font-bold text-sage-ink-muted">Quick actions</p>
    <div class="grid grid-cols-4 gap-3">
      {#each actions as action (action.label)}
        <button
          class="motion flex flex-col items-center gap-2 rounded-card bg-sage-deep px-3 py-4 text-sm font-bold text-sage-ink active:scale-[0.98]"
        >
          <action.icon size={22} />
          {action.label}
        </button>
      {/each}
    </div>

    <div class="mt-auto flex items-center justify-end gap-3">
      {#if app.running}
        <button
          class="motion flex items-center gap-2 rounded-card bg-sage-deep px-5 py-4 font-bold text-sage-ink active:scale-[0.98]"
          onclick={() => (app.running = false)}
        >
          <Square size={18} /> Stop
        </button>
        <div class="flex items-center gap-3 rounded-card bg-sage-deep px-10 py-4 text-lg font-bold text-sage-ink" role="status">
          Running
        </div>
      {:else}
        <button
          class="motion flex items-center gap-3 rounded-card bg-rose px-12 py-4 text-lg font-bold text-rose-on active:scale-[0.98]"
          onclick={() => (app.running = true)}
        >
          <Play size={22} fill="currentColor" /> Launch
        </button>
      {/if}
    </div>
  {/if}
</main>
