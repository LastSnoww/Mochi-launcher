// Phase 1 mock data. Replaced by real instance storage in phase 4.

export type InstanceType = 'vanilla' | 'modded' | 'pvp';
export type Loader = 'fabric' | 'forge' | 'neoforge';
export type ModeId = 'crystal' | 'sword';

export interface Instance {
  id: string;
  name: string;
  type: InstanceType;
  mode?: ModeId;
  mcVersion: string;
  loader?: Loader;
  modCount: number;
  clientMod: boolean;
}

export const loaderNames: Record<Loader, string> = {
  fabric: 'Fabric',
  forge: 'Forge',
  neoforge: 'NeoForge',
};

export const mockInstances: Instance[] = [
  { id: 'crystal', name: 'Crystal practice', type: 'pvp', mode: 'crystal', mcVersion: '1.21.11', loader: 'fabric', modCount: 9, clientMod: true },
  { id: 'sword', name: 'Sword duels', type: 'pvp', mode: 'sword', mcVersion: '26.1', loader: 'fabric', modCount: 6, clientMod: false },
  { id: 'vanilla', name: 'Survival', type: 'vanilla', mcVersion: '26.1', modCount: 0, clientMod: false },
  { id: 'modded', name: 'Tech pack', type: 'modded', mcVersion: '1.21.1', loader: 'neoforge', modCount: 42, clientMod: false },
];
