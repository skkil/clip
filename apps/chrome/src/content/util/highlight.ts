const PROPERTY = '--clip-highlight-color';

export function applyHighlightColor(color: string): void {
  document.documentElement.style.setProperty(PROPERTY, color);
}

export function clearHighlightColor(): void {
  document.documentElement.style.removeProperty(PROPERTY);
}
