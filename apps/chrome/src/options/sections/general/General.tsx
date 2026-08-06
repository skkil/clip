import { useEffect, useState } from 'react';

import { KeyCapture } from '@/options/components/key-capture/KeyCapture';
import { Setting, SettingGroup } from '@/options/components/setting/Setting';
import {
  DEFAULT_PREFERENCES,
  getPreferences,
  setPreferences,
} from '@/shared/preferences';
import type { Key } from '@/shared/types/key';

import styles from './General.module.css';

export function General() {
  const [leaderKey, setLeaderKey] = useState(DEFAULT_PREFERENCES.keys.leader);
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    getPreferences().then((preferences) => {
      setLeaderKey(preferences.keys.leader);
    });
  }, []);

  const handleChangeLeaderKey = async (key: Key) => {
    setLeaderKey(key);
    await setPreferences({ keys: { leader: key } });
    setSaved(true);
  };

  return (
    <section className={styles.section}>
      <div>
        <h2 className={styles.heading}>General</h2>
        <p className={styles.subheading}>
          Control how CLIP behaves while you browse.
        </p>
      </div>

      <SettingGroup>
        <Setting
          label="Leader key"
          description="Hold this key over an element to enter copy mode."
        >
          <KeyCapture value={leaderKey} onChange={handleChangeLeaderKey} />
        </Setting>
      </SettingGroup>

      <div className={styles.status}>{saved && 'Preferences saved.'}</div>
    </section>
  );
}
