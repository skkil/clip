import { useEffect, useState } from 'react';

import { previewToast } from '@/content/util/toast';
import { ColorPicker } from '@/options/components/color-picker/ColorPicker';
import { KeyCapture } from '@/options/components/key-capture/KeyCapture';
import { Setting, SettingGroup } from '@/options/components/setting/Setting';
import { Switch } from '@/shared/components/ui/switch';
import {
  DEFAULT_PREFERENCES,
  getPreferences,
  setPreferences,
} from '@/shared/preferences';
import type { Key } from '@/shared/types/key';

import styles from './General.module.css';

export function General() {
  const [enabled, setEnabled] = useState(DEFAULT_PREFERENCES.enabled);
  const [leaderKey, setLeaderKey] = useState(DEFAULT_PREFERENCES.keys.leader);
  const [toastsEnabled, setToastsEnabled] = useState(
    DEFAULT_PREFERENCES.toasts.enabled,
  );
  const [highlightColor, setHighlightColor] = useState(
    DEFAULT_PREFERENCES.highlight.color,
  );
  const [saved, setSaved] = useState(false);

  useEffect(() => {
    getPreferences().then((preferences) => {
      setEnabled(preferences.enabled);
      setLeaderKey(preferences.keys.leader);
      setToastsEnabled(preferences.toasts.enabled);
      setHighlightColor(preferences.highlight.color);
    });
  }, []);

  const handleChangeEnabled = async (next: boolean) => {
    setEnabled(next);
    await setPreferences({ enabled: next });
    setSaved(true);
  };

  const handleChangeLeaderKey = async (key: Key) => {
    setLeaderKey(key);
    await setPreferences({ keys: { leader: key } });
    setSaved(true);
  };

  const handleChangeToasts = async (enabled: boolean) => {
    setToastsEnabled(enabled);
    await setPreferences({ toasts: { enabled } });
    setSaved(true);
  };

  const handleChangeHighlightColor = async (color: string) => {
    setHighlightColor(color);
    await setPreferences({ highlight: { color } });
    setSaved(true);
  };

  const handleClickPreview = () => {
    previewToast('success', 'Copied');
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
          label="Enable CLIP"
          description="When off, CLIP detaches every page listener and does nothing until you turn it back on."
        >
          <Switch
            checked={enabled}
            label="Enable CLIP"
            onChange={handleChangeEnabled}
          />
        </Setting>

        <Setting
          label="Leader key"
          description="Hold this key over an element to enter copy mode."
        >
          <KeyCapture value={leaderKey} onChange={handleChangeLeaderKey} />
        </Setting>

        <Setting
          label="Highlight color"
          description="The outline drawn around the element you are about to copy."
        >
          <div className={styles.controls}>
            <span
              className={styles.highlightPreview}
              style={{ outlineColor: highlightColor }}
            >
              Preview
            </span>
            <ColorPicker
              value={highlightColor}
              label="Highlight color"
              onChange={handleChangeHighlightColor}
            />
          </div>
        </Setting>

        <Setting
          label="Copy confirmations"
          description="Show a brief message after a copy succeeds or fails. Preview works even while this is off."
        >
          <div className={styles.controls}>
            <button className={styles.preview} onClick={handleClickPreview}>
              Preview
            </button>
            <Switch
              checked={toastsEnabled}
              label="Copy confirmations"
              onChange={handleChangeToasts}
            />
          </div>
        </Setting>
      </SettingGroup>

      <div className={styles.status}>{saved && 'Preferences saved.'}</div>
    </section>
  );
}
