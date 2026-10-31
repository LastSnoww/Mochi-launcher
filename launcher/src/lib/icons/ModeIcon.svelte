<script lang="ts">
  import { Box, Puzzle } from '@lucide/svelte';
  import type { Instance } from '../mock';

  // Our own flat glyphs. PvP mode icons will come from the presets folder in phase 5.
  let { instance, size = 24 }: { instance: Instance; size?: number } = $props();
</script>

{#if instance.type === 'pvp' && instance.mode === 'crystal'}
  <svg width={size} height={size} viewBox="0 0 24 24" aria-hidden="true">
    <path d="M12 2.5 21 12l-9 9.5L3 12Z" fill="var(--color-rose)" />
    <path d="M12 2.5 21 12l-9-3-9 3Z" fill="var(--color-blush)" />
    <path
      d="M12 2.5 21 12l-9 9.5L3 12Zm0 0V9m-9 3 9-3 9 3M12 9v12.5"
      fill="none"
      stroke="var(--color-rose-ink)"
      stroke-width="1.5"
      stroke-linejoin="round"
    />
  </svg>
{:else if instance.type === 'pvp' && instance.mode === 'sword'}
  <svg width={size} height={size} viewBox="0 0 24 24" aria-hidden="true">
    <path
      d="M20.5 3.5v3.2l-9.8 9.8-3.2-3.2 9.8-9.8Z"
      fill="var(--color-blush)"
      stroke="var(--color-rose-ink)"
      stroke-width="1.5"
      stroke-linejoin="round"
    />
    <path d="m6 12 6 6" stroke="var(--color-rose-ink)" stroke-width="2.5" stroke-linecap="round" />
    <path d="m8.6 15.4-4 4" stroke="var(--color-rose)" stroke-width="2.5" stroke-linecap="round" />
    <circle cx="3.9" cy="20.1" r="1.6" fill="var(--color-rose-ink)" />
  </svg>
{:else if instance.type === 'modded'}
  <Puzzle size={size} strokeWidth={2} color="var(--color-rose-ink)" aria-hidden="true" />
{:else}
  <Box size={size} strokeWidth={2} color="var(--color-rose-ink)" aria-hidden="true" />
{/if}
