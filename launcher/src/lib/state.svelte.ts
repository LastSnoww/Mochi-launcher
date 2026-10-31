import { mockInstances, type Instance } from './mock';

class AppState {
  instances = $state<Instance[]>(mockInstances);
  selectedId = $state(mockInstances[0]!.id);
  running = $state(false);
  minimized = $state(false);

  selected = $derived(this.instances.find((i) => i.id === this.selectedId) ?? this.instances[0]);

  /** Animations pause while the game runs or the window is minimized (spec section 5). */
  paused = $derived(this.running || this.minimized);
}

export const app = new AppState();
