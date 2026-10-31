<script lang="ts">
  import { Plus, Settings, UserRound } from '@lucide/svelte';
  import ModeIcon from '../icons/ModeIcon.svelte';
  import { app } from '../state.svelte';
</script>

<aside class="flex w-70 shrink-0 flex-col gap-2 bg-blush px-4 pb-4">
  {#if app.selected}
    <p class="mt-2 text-xs font-bold text-rose-ink-muted">Selected instance</p>
    <div class="flex items-center gap-3 rounded-card bg-blush-deep p-3">
      <div class="grid size-11 shrink-0 place-items-center rounded-card bg-blush">
        <ModeIcon instance={app.selected} size={26} />
      </div>
      <div class="min-w-0">
        <p class="truncate font-bold text-rose-ink">{app.selected.name}</p>
        <p class="text-sm text-rose-ink-muted">Minecraft {app.selected.mcVersion}</p>
      </div>
    </div>
  {/if}

  <p class="mt-4 text-xs font-bold text-rose-ink-muted">Instances</p>
  <ul class="-mx-1 flex min-h-0 flex-1 flex-col gap-1 overflow-y-auto px-1 py-1">
    {#each app.instances as instance (instance.id)}
      {@const selected = instance.id === app.selectedId}
      <li>
        <button
          class={[
            'flex w-full items-center gap-3 rounded-card px-3 py-2 text-left hover:bg-blush-deep',
            selected && 'bg-blush-deep',
          ]}
          aria-current={selected}
          onclick={() => (app.selectedId = instance.id)}
        >
          <ModeIcon {instance} size={20} />
          <span class="min-w-0 flex-1 truncate text-sm font-bold text-rose-ink">{instance.name}</span>
          <span class="text-xs text-rose-ink-muted">{instance.mcVersion}</span>
        </button>
      </li>
    {/each}
  </ul>

  <button
    class="motion flex items-center justify-center gap-2 rounded-card border-2 border-rose-ink-muted px-3 py-2.5 text-sm font-bold text-rose-ink hover:bg-blush-deep active:scale-[0.98]"
  >
    <Plus size={18} /> New instance
  </button>

  <div class="mt-2 flex items-center gap-2">
    <button class="flex min-w-0 flex-1 items-center gap-3 rounded-card px-2 py-2 text-left hover:bg-blush-deep">
      <span class="grid size-8 shrink-0 place-items-center rounded-chip bg-blush-deep text-rose-ink">
        <UserRound size={18} />
      </span>
      <span class="min-w-0">
        <span class="block truncate text-sm font-bold text-rose-ink">Not signed in</span>
        <span class="block text-xs text-rose-ink-muted">Add a Microsoft account</span>
      </span>
    </button>
    <button class="grid size-10 place-items-center rounded-card text-rose-ink hover:bg-blush-deep" aria-label="Settings">
      <Settings size={20} />
    </button>
  </div>
</aside>
