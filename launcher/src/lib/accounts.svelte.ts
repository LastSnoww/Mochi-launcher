import { isTauri } from '@tauri-apps/api/core';
import { accountsApi, errorMessage, isCommandError, type Account, type AccountsView, type DeviceCode } from './api/accounts';

export type LoginState =
  | { phase: 'idle' }
  | { phase: 'requesting' }
  | { phase: 'code'; code: DeviceCode }
  | { phase: 'signing-in' }
  | { phase: 'error'; message: string };

class AccountsStore {
  list = $state<Account[]>([]);
  active = $state<string | null>(null);
  /** Skin data URLs keyed by `uuid:skinUrl`, so a changed skin is fetched again. */
  heads = $state<Record<string, string | null>>({});
  /** Accounts whose refresh token no longer works. */
  expired = $state<Record<string, boolean>>({});
  login = $state<LoginState>({ phase: 'idle' });
  error = $state<string | null>(null);

  current = $derived(this.list.find((a) => a.uuid === this.active));

  headFor(account: Account): string | null | undefined {
    return this.heads[`${account.uuid}:${account.skinUrl}`];
  }

  async init() {
    if (!isTauri()) return;
    await accountsApi.onLogin(async (event) => {
      if (event.status === 'signing-in') this.login = { phase: 'signing-in' };
      else if (event.status === 'error') this.login = { phase: 'error', message: event.message };
      else {
        this.login = { phase: 'idle' };
        this.expired[event.uuid] = false;
        this.apply(await accountsApi.list());
      }
    });
    this.apply(await accountsApi.list());
    if (this.active) void this.refresh(this.active);
  }

  private apply(view: AccountsView) {
    this.list = view.accounts;
    this.active = view.active;
    for (const account of view.accounts) {
      const key = `${account.uuid}:${account.skinUrl}`;
      if (!(key in this.heads)) {
        this.heads[key] = null;
        accountsApi.head(account.uuid).then(
          (src) => (this.heads[key] = src),
          () => (this.heads[key] = null),
        );
      }
    }
  }

  /** Silent refresh: picks up name and skin changes and detects revoked sign-ins. */
  async refresh(uuid: string) {
    try {
      this.apply(await accountsApi.refresh(uuid));
      this.expired[uuid] = false;
    } catch (err) {
      if (isCommandError(err) && err.kind === 'session-expired') this.expired[uuid] = true;
    }
  }

  async switchTo(uuid: string) {
    await this.run(() => accountsApi.switch(uuid));
  }

  async remove(uuid: string) {
    await this.run(() => accountsApi.remove(uuid));
  }

  async startLogin() {
    this.login = { phase: 'requesting' };
    try {
      this.login = { phase: 'code', code: await accountsApi.loginStart() };
    } catch (err) {
      this.login = { phase: 'error', message: errorMessage(err) };
    }
  }

  async cancelLogin() {
    this.login = { phase: 'idle' };
    await accountsApi.loginCancel();
  }

  async openLoginPage() {
    try {
      await accountsApi.openVerification();
    } catch (err) {
      this.login = { phase: 'error', message: errorMessage(err) };
    }
  }

  private async run(action: () => Promise<AccountsView>) {
    this.error = null;
    try {
      this.apply(await action());
    } catch (err) {
      this.error = errorMessage(err);
    }
  }
}

export const accounts = new AccountsStore();
