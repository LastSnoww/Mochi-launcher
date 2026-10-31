<script lang="ts">
  import { isTauri } from '@tauri-apps/api/core';
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { Copy, Minus, Square, X } from '@lucide/svelte';
  import MochiLogo from '../icons/MochiLogo.svelte';
  import { app } from '../state.svelte';

  // Outside Tauri (plain `pnpm dev` in a browser) the window API is unavailable.
  const win = isTauri() ? getCurrentWindow() : null;
  let maximized = $state(false);

  $effect(() => {
    if (!win) return;
    let unlisten: (() => void) | undefined;
    const sync = async () => {
      maximized = await win.isMaximized();
      app.minimized = await win.isMinimized();
    };
    sync();
    win.onResized(sync).then((fn) => (unlisten = fn));
    return () => unlisten?.();
  });

  const buttonClass =
    'grid h-8 w-10 place-items-center rounded-chip text-rose-ink hover:bg-blush-deep motion active:scale-95';
</script>

<header data-tauri-drag-region class="flex h-10 shrink-0 items-center gap-2 bg-blush pr-1 pl-4">
  <div class="pointer-events-none flex items-center gap-2">
    <MochiLogo size={18} />
    <span class="text-sm font-bold text-rose-ink">Mochi Client</span>
  </div>
  <div data-tauri-drag-region class="h-full flex-1"></div>
  <button class={buttonClass} aria-label="Minimize" onclick={() => win?.minimize()}>
    <Minus size={16} />
  </button>
  <button class={buttonClass} aria-label={maximized ? 'Restore' : 'Maximize'} onclick={() => win?.toggleMaximize()}>
    {#if maximized}<Copy size={14} />{:else}<Square size={14} />{/if}
  </button>
  <button class={buttonClass} aria-label="Close" onclick={() => win?.close()}>
    <X size={16} />
  </button>
</header>
