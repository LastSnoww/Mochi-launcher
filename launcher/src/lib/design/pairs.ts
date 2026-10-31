/**
 * Every foreground/background token combination the UI uses.
 * Add a pair here before using a new combination; the contrast test enforces it.
 */
export type Pair = { fg: string; bg: string; kind: 'text' | 'non-text' };

export const pairs: Pair[] = [
  // Sidebar and title bar
  { fg: 'rose-ink', bg: 'blush', kind: 'text' },
  { fg: 'rose-ink-muted', bg: 'blush', kind: 'text' },
  { fg: 'rose-ink', bg: 'blush-deep', kind: 'text' },
  { fg: 'rose-ink-muted', bg: 'blush-deep', kind: 'text' },
  { fg: 'danger', bg: 'blush', kind: 'text' },
  { fg: 'danger', bg: 'blush-deep', kind: 'text' },

  // Primary action
  { fg: 'rose-on', bg: 'rose', kind: 'text' },

  // Main area
  { fg: 'sage-ink', bg: 'sage', kind: 'text' },
  { fg: 'sage-ink-muted', bg: 'sage', kind: 'text' },
  { fg: 'sage-ink', bg: 'sage-deep', kind: 'text' },
  { fg: 'danger', bg: 'sage', kind: 'text' },
  { fg: 'danger', bg: 'sage-deep', kind: 'text' },

  // Focus ring (WCAG 1.4.11 / 2.4.13 need 3:1)
  { fg: 'rose-ink', bg: 'blush', kind: 'non-text' },
  { fg: 'rose-ink', bg: 'blush-deep', kind: 'non-text' },
  { fg: 'rose-ink', bg: 'sage', kind: 'non-text' },
  { fg: 'rose-ink', bg: 'sage-deep', kind: 'non-text' },
];
