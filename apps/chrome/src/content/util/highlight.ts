import {
  DEFAULT_PREFERENCES,
  getPreferences,
  registerPreferencesChangeEventListener,
} from '@/shared/preferences';

const PROPERTY = '--clip-highlight-color';

function apply(color: string): void {
  document.documentElement.style.setProperty(PROPERTY, color);
}

export function loadHighlightColor(): void {
  apply(DEFAULT_PREFERENCES.highlight.color);

  getPreferences().then((preferences) => {
    apply(preferences.highlight.color);
  });

  registerPreferencesChangeEventListener((preferences) => {
    apply(preferences.highlight.color);
  });
}
