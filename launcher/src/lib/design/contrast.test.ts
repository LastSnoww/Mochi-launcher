import { readFileSync, readdirSync } from 'node:fs';
import { join } from 'node:path';
import { describe, expect, it } from 'vitest';
import { contrastRatio, parseColorTokens } from './contrast';
import { pairs } from './pairs';

const srcDir = join(import.meta.dirname, '..', '..');
const tokens = parseColorTokens(readFileSync(join(srcDir, 'app.css'), 'utf8'));

function svelteFiles(dir: string): string[] {
  return readdirSync(dir, { withFileTypes: true }).flatMap((e) =>
    e.isDirectory() ? svelteFiles(join(dir, e.name)) : e.name.endsWith('.svelte') ? [join(dir, e.name)] : [],
  );
}
const components = svelteFiles(srcDir).map((path) => ({ path, source: readFileSync(path, 'utf8') }));

describe('design tokens', () => {
  it('parses every token from app.css', () => {
    expect([...tokens.keys()].sort()).toEqual(
      [
        'blush', 'blush-deep', 'rose', 'rose-ink', 'rose-ink-muted', 'rose-on',
        'sage', 'sage-deep', 'sage-ink', 'sage-ink-muted', 'danger',
      ].sort(),
    );
  });

  it.each(pairs)('$fg on $bg ($kind) meets WCAG AA', ({ fg, bg, kind }) => {
    const ratio = contrastRatio(tokens.get(fg)!, tokens.get(bg)!);
    expect(ratio, `${fg} on ${bg} is ${ratio.toFixed(2)}:1`).toBeGreaterThanOrEqual(kind === 'text' ? 4.5 : 3);
  });

  it('every text colour used in components is declared in a checked pair', () => {
    const declared = new Set(pairs.filter((p) => p.kind === 'text').map((p) => p.fg));
    const names = [...tokens.keys()].join('|');
    // Token names contain hyphens, so match the whole class name rather than relying on \b.
    const classRe = new RegExp(`(?<![\\w-])text-(${names})(?![\\w-])`, 'g');
    const used = new Set<string>();
    for (const { source } of components) {
      for (const m of source.matchAll(classRe)) used.add(m[1]!);
    }
    expect([...used].filter((t) => !declared.has(t))).toEqual([]);
  });

  it('components use tokens, never raw hex colours', () => {
    const offenders = components.filter(({ source }) => /#[0-9a-fA-F]{3,8}\b/.test(source)).map((c) => c.path);
    expect(offenders).toEqual([]);
  });
});
