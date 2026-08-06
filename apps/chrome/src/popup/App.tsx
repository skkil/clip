import { useEffect, useState } from 'react';

import { Kbd } from '@/shared/components/ui/kbd';
import { Logo } from '@/shared/components/ui/logo';
import { SettingsIcon } from '@/shared/components/ui/settings-icon';
import { DEFAULT_PREFERENCES, getPreferences } from '@/shared/preferences';
import { getKeyLabel } from '@/shared/utils/key';

import styles from './App.module.css';

export default function App() {
  const [leaderKey, setLeaderKey] = useState(DEFAULT_PREFERENCES.keys.leader);

  useEffect(() => {
    getPreferences().then((preferences) => {
      setLeaderKey(preferences.keys.leader);
    });
  }, []);

  const leader = getKeyLabel(leaderKey);

  const handleClickSettingsButton = () => {
    chrome.runtime.openOptionsPage();
  };

  const handleClickGuideButton = () => {
    chrome.tabs.create({ url: chrome.runtime.getURL('welcome.html') });
  };

  const shortcuts = [
    { label: 'Copy an element', keys: [leader, 'Click'] },
    { label: 'Pick a format', keys: [leader, 'Shift'] },
    { label: 'Capture a region', keys: [leader, 'S'] },
  ];

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

      <p className={styles.hint}>
        Hold <Kbd>{leader}</Kbd> on any page, then click what you want to copy.
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

      <footer className={styles.footer}>
        <button className={styles.guide} onClick={handleClickGuideButton}>
          Show me how
        </button>
      </footer>
    </div>
  );
}
