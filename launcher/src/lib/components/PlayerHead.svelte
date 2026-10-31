<script lang="ts">
  import { UserRound } from '@lucide/svelte';

  // Draws the face and hat layer of the player's own skin, scaled up without smoothing.
  let {
    src,
    size = 32,
    surface = 'blush',
  }: { src: string | null | undefined; size?: number; surface?: 'blush' | 'sage' } = $props();

  let canvas = $state<HTMLCanvasElement>();
  let failed = $state(false);

  $effect(() => {
    failed = false;
    if (!src || !canvas) return;
    const target = canvas;
    const img = new Image();
    img.onload = () => {
      const ctx = target.getContext('2d');
      if (!ctx) return;
      ctx.imageSmoothingEnabled = false;
      ctx.clearRect(0, 0, 8, 8);
      ctx.drawImage(img, 8, 8, 8, 8, 0, 0, 8, 8);
      ctx.drawImage(img, 40, 8, 8, 8, 0, 0, 8, 8);
    };
    img.onerror = () => (failed = true);
    img.src = src;
  });
</script>

{#if src && !failed}
  <canvas
    bind:this={canvas}
    width="8"
    height="8"
    class="shrink-0 rounded-chip [image-rendering:pixelated]"
    style:width="{size}px"
    style:height="{size}px"
    aria-hidden="true"
  ></canvas>
{:else}
  <span
    class={[
      'grid shrink-0 place-items-center rounded-chip text-rose-ink',
      surface === 'blush' ? 'bg-blush-deep' : 'bg-blush',
    ]}
    style:width="{size}px"
    style:height="{size}px"
    aria-hidden="true"
  >
    <UserRound size={Math.round(size * 0.56)} />
  </span>
{/if}
