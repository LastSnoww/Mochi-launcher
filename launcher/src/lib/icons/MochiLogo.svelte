<script lang="ts">
  let { size = 20 }: { size?: number } = $props();
</script>

<svg width={size} height={size} viewBox="0 0 32 32" aria-hidden="true">
  <rect width="32" height="32" rx="7" fill="var(--color-sage)" />
  <path
    d="M6.5 22.5C6.5 14 10.8 9.5 16 9.5s9.5 4.5 9.5 13c0 1-.6 1.6-1.6 1.6H8.1c-1 0-1.6-.6-1.6-1.6Z"
    fill="var(--color-blush)"
  />
  <circle cx="13" cy="17.5" r="1" fill="var(--color-rose-ink)" />
  <circle cx="19" cy="17.5" r="1" fill="var(--color-rose-ink)" />
  <ellipse cx="11" cy="19.5" rx="1.4" ry=".8" fill="var(--color-rose)" />
  <ellipse cx="21" cy="19.5" rx="1.4" ry=".8" fill="var(--color-rose)" />
</svg>
