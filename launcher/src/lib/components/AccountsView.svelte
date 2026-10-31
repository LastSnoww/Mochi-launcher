<script lang="ts">
  import { ArrowLeft, Check, Copy, ExternalLink, LoaderCircle, Plus, Trash2 } from '@lucide/svelte';
  import { accounts } from '../accounts.svelte';
  import { app } from '../state.svelte';
  import PlayerHead from './PlayerHead.svelte';

  let confirmRemove = $state<string | null>(null);
  let copied = $state(false);

  async function copyCode(code: string) {
    try {
      await navigator.clipboard.writeText(code);
      copied = true;
      setTimeout(() => (copied = false), 1500);
    } catch {
      // Clipboard access can be refused; the code is still visible to type by hand.
    }
  }

  const secondaryButton =
    'motion flex items-center gap-2 rounded-card bg-sage-deep px-4 py-2.5 text-sm font-bold text-sage-ink active:scale-[0.98]';
  const primaryButton =
    'motion flex items-center gap-2 rounded-card bg-rose px-5 py-3 font-bold text-rose-on active:scale-[0.98]';
</script>

<main class="flex min-w-0 flex-1 flex-col gap-2 overflow-y-auto bg-sage p-6">
  <div class="flex items-center gap-3">
    <button
      class="grid size-9 place-items-center rounded-card text-sage-ink hover:bg-sage-deep"
      aria-label="Back"
      onclick={() => (app.view = 'home')}
    >
      <ArrowLeft size={20} />
    </button>
    <h1 class="text-2xl font-bold text-sage-ink">Accounts</h1>
  </div>

  {#if accounts.list.length > 0}
    <p class="mt-4 text-xs font-bold text-sage-ink-muted">Microsoft accounts</p>
    <ul class="flex flex-col gap-2">
      {#each accounts.list as account (account.uuid)}
        {@const active = account.uuid === accounts.active}
        <li class="flex items-center gap-4 rounded-card bg-sage-deep p-4">
          <PlayerHead src={accounts.headFor(account)} size={40} surface="sage" />
          <div class="min-w-0 flex-1">
            <p class="truncate font-bold text-sage-ink">{account.name}</p>
            {#if accounts.expired[account.uuid]}
              <p class="text-sm font-bold text-danger">Signed out. Add it again below to sign back in.</p>
            {:else if active}
              <p class="text-sm text-sage-ink">In use</p>
            {/if}
          </div>
          {#if !active}
            <button class="motion rounded-card bg-sage px-3 py-2 text-sm font-bold text-sage-ink active:scale-[0.98]" onclick={() => accounts.switchTo(account.uuid)}>
              Use
            </button>
          {/if}
          {#if confirmRemove === account.uuid}
            <button
              class="motion rounded-card bg-sage px-3 py-2 text-sm font-bold text-danger active:scale-[0.98]"
              onclick={() => {
                confirmRemove = null;
                accounts.remove(account.uuid);
              }}
            >
              Remove {account.name}?
            </button>
            <button class="rounded-card px-3 py-2 text-sm font-bold text-sage-ink hover:bg-sage" onclick={() => (confirmRemove = null)}>
              Keep
            </button>
          {:else}
            <button
              class="grid size-9 place-items-center rounded-card text-sage-ink hover:bg-sage"
              aria-label="Remove {account.name}"
              onclick={() => (confirmRemove = account.uuid)}
            >
              <Trash2 size={18} />
            </button>
          {/if}
        </li>
      {/each}
    </ul>
  {/if}

  {#if accounts.error}
    <p class="text-sm font-bold text-danger" role="alert">{accounts.error}</p>
  {/if}

  <p class="mt-4 text-xs font-bold text-sage-ink-muted">Add an account</p>
  <section class="flex flex-col items-start gap-4 rounded-card bg-sage-deep p-5" aria-live="polite">
    {#if accounts.login.phase === 'idle'}
      <p class="text-sage-ink">Sign in with the Microsoft account that owns Minecraft: Java Edition.</p>
      <button class={primaryButton} onclick={() => accounts.startLogin()}>
        <Plus size={18} /> Add Microsoft account
      </button>
    {:else if accounts.login.phase === 'requesting'}
      <p class="flex items-center gap-2 text-sage-ink"><LoaderCircle size={18} class="animate-spin" /> Getting a sign-in code…</p>
    {:else if accounts.login.phase === 'code'}
      {@const code = accounts.login.code}
      <p class="text-sage-ink">Open the Microsoft sign-in page and enter this code:</p>
      <div class="flex items-center gap-3">
        <span class="rounded-card bg-sage px-4 py-2 text-3xl font-bold tracking-widest text-sage-ink select-text">{code.userCode}</span>
        <button class={secondaryButton} onclick={() => copyCode(code.userCode)}>
          {#if copied}<Check size={16} /> Copied{:else}<Copy size={16} /> Copy{/if}
        </button>
      </div>
      <div class="flex items-center gap-3">
        <button class={primaryButton} onclick={() => accounts.openLoginPage()}>
          <ExternalLink size={18} /> Open Microsoft sign-in
        </button>
        <button class={secondaryButton} onclick={() => accounts.cancelLogin()}>Cancel</button>
      </div>
      <p class="text-sm text-sage-ink">
        Waiting for you to finish in the browser. The code works for {Math.round(code.expiresIn / 60)} minutes.
      </p>
    {:else if accounts.login.phase === 'signing-in'}
      <p class="flex items-center gap-2 text-sage-ink"><LoaderCircle size={18} class="animate-spin" /> Signing in to Minecraft…</p>
    {:else}
      <p class="font-bold text-danger" role="alert">{accounts.login.message}</p>
      <button class={primaryButton} onclick={() => accounts.startLogin()}>Try again</button>
    {/if}
  </section>
</main>
