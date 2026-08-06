import { useEffect, useState } from 'react';

import { Kbd } from '@/shared/components/ui/kbd';
import { Logo } from '@/shared/components/ui/logo';
import { SettingsIcon } from '@/shared/components/ui/settings-icon';
import { Switch } from '@/shared/components/ui/switch';
import {
  type Preferences,
  getPreferences,
  setPreferences,
} from '@/shared/preferences';
import {
  type Site,
  getSite,
  isEnabledOn,
  setSiteEnabled,
} from '@/shared/sites';
import { getKeyLabel } from '@/shared/utils/key';

import styles from './App.module.css';

export default function App() {
  const [preferences, setLoadedPreferences] = useState<Preferences | null>(
    null,
  );
  const [site, setSite] = useState<Site | null>(null);

  useEffect(() => {
    Promise.all([
      getPreferences(),
      chrome.tabs.query({ active: true, currentWindow: true }),
    ]).then(([loaded, tabs]) => {
      setLoadedPreferences(loaded);
      setSite(getSite(tabs[0]?.url));
    });
  }, []);

  const handleClickSettingsButton = () => {
    chrome.runtime.openOptionsPage();
  };

  const handleClickGuideButton = () => {
    chrome.tabs.create({ url: chrome.runtime.getURL('welcome.html') });
  };

  const handleToggleExtension = (enabled: boolean) => {
    if (!preferences) {
      return;
    }

    setLoadedPreferences({ ...preferences, enabled });
    void setPreferences({ enabled });
  };

  const handleToggleSite = (enabled: boolean) => {
    if (!preferences || !site?.host) {
      return;
    }

    const sites = setSiteEnabled(preferences.sites, site.host, enabled);
    setLoadedPreferences({ ...preferences, sites });
    void setPreferences({ sites });
  };

  if (!preferences || !site) {
    return <div className={styles.popup} />;
  }

  const leader = getKeyLabel(preferences.keys.leader);
  const active = isEnabledOn(preferences, site);
  const allowedHere = isEnabledOn({ ...preferences, enabled: true }, site);

  const shortcuts = [
    { label: 'Copy an element', keys: [leader, 'Click'] },
    { label: 'Pick a format', keys: [leader, 'Shift'] },
    { label: 'Capture a region', keys: [leader, 'S'] },
  ];

  const pausedReason = !site.supported
    ? 'CLIP cannot run on this kind of page.'
    : !preferences.enabled
      ? 'CLIP is off on every page. It is not listening for anything.'
      : `CLIP is off on ${site.host}. No listeners are attached here.`;

  return (
    <div className={styles.popup}>
      <header className={styles.header}>
        <span className={styles.mark}>
          <Logo size={20} />
        </span>

        <h1 className={styles.wordmark}>CLIP</h1>

        <button
          className={styles.settings}
          onClick={handleClickSettingsButton}
          aria-label="Settings"
          title="Settings"
        >
          <SettingsIcon />
        </button>
      </header>

      <div className={styles.toggles}>
        <div className={styles.toggle}>
          <span className={styles.toggleLabel}>
            {preferences.enabled ? 'On' : 'Off everywhere'}
          </span>
          <Switch
            checked={preferences.enabled}
            label="Enable CLIP"
            onChange={handleToggleExtension}
          />
        </div>

        {site.host && (
          <div
            className={`${styles.toggle} ${
              preferences.enabled ? '' : styles.toggleDimmed
            }`}
          >
            <span className={styles.toggleLabel}>
              Run on <span className={styles.host}>{site.host}</span>
            </span>
            <Switch
              checked={allowedHere}
              disabled={!preferences.enabled}
              label={`Run CLIP on ${site.host}`}
              onChange={handleToggleSite}
            />
          </div>
        )}
      </div>

      {active ? (
        <>
          <p className={styles.hint}>
            Hold <Kbd>{leader}</Kbd> on any page, then click what you want to
            copy.
          </p>

          <ul className={styles.shortcuts}>
            {shortcuts.map(({ label, keys }) => (
              <li key={label} className={styles.shortcut}>
                <span className={styles.shortcutLabel}>{label}</span>
                <span className={styles.shortcutKeys}>
                  {keys.map((key, index) => (
                    <span key={key} className={styles.shortcutKey}>
                      {index > 0 && <span className={styles.plus}>+</span>}
                      <Kbd>{key}</Kbd>
                    </span>
                  ))}
                </span>
              </li>
            ))}
          </ul>
        </>
      ) : (
        <p className={styles.paused}>{pausedReason}</p>
      )}

      <footer className={styles.footer}>
        <button className={styles.guide} onClick={handleClickGuideButton}>
          Show me how
        </button>
      </footer>
    </div>
  );
}
