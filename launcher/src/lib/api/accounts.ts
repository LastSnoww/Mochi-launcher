import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

// Mirrors src-tauri/src/auth/commands.rs.

export interface Account {
  uuid: string;
  name: string;
  skinUrl: string | null;
}

export interface AccountsView {
  active: string | null;
  accounts: Account[];
}

export interface DeviceCode {
  userCode: string;
  verificationUri: string;
  expiresIn: number;
}

export type LoginEvent =
  | { status: 'signing-in' }
  | { status: 'done'; uuid: string }
  | { status: 'error'; message: string };

export interface CommandError {
  kind: 'session-expired' | 'network' | 'error';
  message: string;
}

export function isCommandError(err: unknown): err is CommandError {
  return typeof err === 'object' && err !== null && 'kind' in err && 'message' in err;
}

export function errorMessage(err: unknown): string {
  return isCommandError(err) ? err.message : String(err);
}

export const accountsApi = {
  list: () => invoke<AccountsView>('accounts_list'),
  switch: (uuid: string) => invoke<AccountsView>('account_switch', { uuid }),
  remove: (uuid: string) => invoke<AccountsView>('account_remove', { uuid }),
  refresh: (uuid: string) => invoke<AccountsView>('account_refresh', { uuid }),
  head: (uuid: string) => invoke<string | null>('account_head', { uuid }),
  loginStart: () => invoke<DeviceCode>('auth_login_start'),
  loginCancel: () => invoke<void>('auth_login_cancel'),
  openVerification: () => invoke<void>('auth_open_verification'),
  onLogin: (handler: (event: LoginEvent) => void): Promise<UnlistenFn> =>
    listen<LoginEvent>('auth:login', (e) => handler(e.payload)),
};
