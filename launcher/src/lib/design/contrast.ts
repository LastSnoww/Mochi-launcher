/** WCAG 2.x relative luminance of a #rrggbb colour. */
export function luminance(hex: string): number {
  const channels = [1, 3, 5].map((i) => parseInt(hex.slice(i, i + 2), 16) / 255);
  const [r, g, b] = channels.map((c) => (c <= 0.03928 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4));
  return 0.2126 * r! + 0.7152 * g! + 0.0722 * b!;
}

export function contrastRatio(a: string, b: string): number {
  const [hi, lo] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return (hi! + 0.05) / (lo! + 0.05);
}

/** Reads `--color-<name>: #hex;` declarations from the `@theme` block of a CSS file. */
export function parseColorTokens(css: string): Map<string, string> {
  const theme = css.match(/@theme\s*\{([\s\S]*?)\n\}/)?.[1] ?? '';
  const tokens = new Map<string, string>();
  for (const m of theme.matchAll(/--color-([a-z-]+):\s*(#[0-9a-fA-F]{6})\s*;/g)) {
    tokens.set(m[1]!, m[2]!.toLowerCase());
  }
  return tokens;
}
