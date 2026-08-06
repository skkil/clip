import { Key } from '@/shared/types/key';

const STORAGE_KEY = 'preferences';

export interface Preferences {
  keys: {
    leader: Key;
  };
  toasts: {
    enabled: boolean;
  };
  highlight: {
    color: string;
  };
}

export const DEFAULT_PREFERENCES: Preferences = {
  keys: {
    leader: Key.ALT,
  },
  toasts: {
    enabled: true,
  },
  highlight: {
    color: '#ff0000',
  },
};

export async function getPreferences(): Promise<Preferences> {
  const result = await chrome.storage.sync.get(STORAGE_KEY);
  return {
    ...DEFAULT_PREFERENCES,
    ...(result[STORAGE_KEY] as Partial<Preferences> | undefined),
  };
}

export async function setPreferences(
  preferences: Partial<Preferences>,
): Promise<void> {
  const current = await getPreferences();
  await chrome.storage.sync.set({
    [STORAGE_KEY]: { ...current, ...preferences },
  });
}

export function registerPreferencesChangeEventListener(
  callback: (preferences: Preferences) => void,
) {
  chrome.storage.onChanged.addListener((changes, areaName) => {
    if (areaName !== 'sync' || !changes[STORAGE_KEY]) {
      return;
    }
    callback({
      ...DEFAULT_PREFERENCES,
      ...(changes[STORAGE_KEY].newValue as Partial<Preferences> | undefined),
    });
  });
}
