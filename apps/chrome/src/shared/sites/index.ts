import { type Preferences, SiteMode } from '@/shared/preferences';

const INTERNAL_PROTOCOL = 'chrome-extension:';
const WEB_PROTOCOLS = ['http:', 'https:'];
const UNSUPPORTED_HOSTS = ['chromewebstore.google.com'];
const HOSTNAME =
  /^[a-z0-9]([a-z0-9-]*[a-z0-9])?(\.[a-z0-9]([a-z0-9-]*[a-z0-9])?)*$/;

export interface Site {
  host: string | null;
  supported: boolean;
}

const UNSUPPORTED: Site = { host: null, supported: false };
const INTERNAL: Site = { host: null, supported: true };

export function getSite(url: string | null | undefined): Site {
  if (!url) {
    return UNSUPPORTED;
  }

  let parsed: URL;
  try {
    parsed = new URL(url);
  } catch {
    return UNSUPPORTED;
  }

  if (parsed.protocol === INTERNAL_PROTOCOL) {
    return INTERNAL;
  }

  if (!WEB_PROTOCOLS.includes(parsed.protocol)) {
    return UNSUPPORTED;
  }

  const host = parsed.hostname.toLowerCase().replace(/\.$/, '');

  if (!host || UNSUPPORTED_HOSTS.includes(host)) {
    return UNSUPPORTED;
  }

  if (host === 'chrome.google.com' && parsed.pathname.startsWith('/webstore')) {
    return UNSUPPORTED;
  }

  return { host, supported: true };
}

export function parseSitePattern(input: string): string | null {
  const trimmed = input.trim().toLowerCase();
  if (!trimmed) {
    return null;
  }

  const withScheme = /^[a-z][a-z0-9+.-]*:\/\//.test(trimmed)
    ? trimmed
    : `https://${trimmed}`;

  let parsed: URL;
  try {
    parsed = new URL(withScheme);
  } catch {
    return null;
  }

  const host = parsed.hostname.replace(/\.$/, '').replace(/^www\./, '');

  return HOSTNAME.test(host) ? host : null;
}

export function matchesSitePattern(host: string, pattern: string): boolean {
  return host === pattern || host.endsWith(`.${pattern}`);
}

export function isEnabledOn(preferences: Preferences, site: Site): boolean {
  if (!preferences.enabled || !site.supported) {
    return false;
  }

  const host = site.host;
  if (host === null) {
    return true;
  }

  const { mode, blocklist, allowlist } = preferences.sites;

  if (mode === SiteMode.ALLOWLIST) {
    return allowlist.some((pattern) => matchesSitePattern(host, pattern));
  }

  return !blocklist.some((pattern) => matchesSitePattern(host, pattern));
}

function addPattern(list: string[], pattern: string): string[] {
  return list.includes(pattern) ? list : [...list, pattern].sort();
}

function removeMatching(list: string[], host: string): string[] {
  return list.filter((pattern) => !matchesSitePattern(host, pattern));
}

export function setSiteEnabled(
  sites: Preferences['sites'],
  host: string,
  enabled: boolean,
): Preferences['sites'] {
  if (sites.mode === SiteMode.ALLOWLIST) {
    return {
      ...sites,
      allowlist: enabled
        ? addPattern(sites.allowlist, host)
        : removeMatching(sites.allowlist, host),
    };
  }

  return {
    ...sites,
    blocklist: enabled
      ? removeMatching(sites.blocklist, host)
      : addPattern(sites.blocklist, host),
  };
}

export function removeSitePattern(
  sites: Preferences['sites'],
  pattern: string,
): Preferences['sites'] {
  const key = sites.mode === SiteMode.ALLOWLIST ? 'allowlist' : 'blocklist';

  return {
    ...sites,
    [key]: sites[key].filter((entry) => entry !== pattern),
  };
}

export function addSitePattern(
  sites: Preferences['sites'],
  pattern: string,
): Preferences['sites'] {
  const key = sites.mode === SiteMode.ALLOWLIST ? 'allowlist' : 'blocklist';

  return {
    ...sites,
    [key]: addPattern(sites[key], pattern),
  };
}
