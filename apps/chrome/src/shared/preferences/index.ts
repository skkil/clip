import { Key } from '@/shared/types/key';

const STORAGE_KEY = 'preferences';

export enum SiteMode {
  BLOCKLIST = 'blocklist',
  ALLOWLIST = 'allowlist',
}

export interface Preferences {
  enabled: boolean;
  sites: {
    mode: SiteMode;
    blocklist: string[];
    allowlist: string[];
  };
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

export type PreferencesUpdate = {
  [K in keyof Preferences]?: Preferences[K] extends object
    ? Partial<Preferences[K]>
    : Preferences[K];
};

export const DEFAULT_PREFERENCES: Preferences = {
  enabled: true,
  sites: {
    mode: SiteMode.BLOCKLIST,
    blocklist: [],
    allowlist: [],
  },
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

function apply(current: Preferences, update: PreferencesUpdate): Preferences {
  return {
    enabled: update.enabled ?? current.enabled,
    sites: { ...current.sites, ...update.sites },
    keys: { ...current.keys, ...update.keys },
    toasts: { ...current.toasts, ...update.toasts },
    highlight: { ...current.highlight, ...update.highlight },
  };
}

function withDefaults(stored: PreferencesUpdate | undefined): Preferences {
  return apply(structuredClone(DEFAULT_PREFERENCES), stored ?? {});
}

export async function getPreferences(): Promise<Preferences> {
  const result = await chrome.storage.sync.get(STORAGE_KEY);
  return withDefaults(result[STORAGE_KEY] as PreferencesUpdate | undefined);
}

export async function setPreferences(update: PreferencesUpdate): Promise<void> {
  const current = await getPreferences();
  await chrome.storage.sync.set({
    [STORAGE_KEY]: apply(current, update),
  });
}

export function registerPreferencesChangeEventListener(
  callback: (preferences: Preferences) => void,
) {
  chrome.storage.onChanged.addListener((changes, areaName) => {
    if (areaName !== 'sync' || !changes[STORAGE_KEY]) {
      return;
    }

    callback(
      withDefaults(
        changes[STORAGE_KEY].newValue as PreferencesUpdate | undefined,
      ),
    );
  });
}
