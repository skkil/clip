import { useEffect, useState } from 'react';

import type { Key } from '@/shared/types/key';
import { getKey, getKeyLabel } from '@/shared/utils/key';

import styles from './KeyCapture.module.css';

interface KeyCaptureProps {
  value: Key;
  onChange: (key: Key) => void;
}

export function KeyCapture({ value, onChange }: KeyCaptureProps) {
  const [capturing, setCapturing] = useState(false);

  useEffect(() => {
    if (!capturing) {
      return;
    }

    const handleKeyDown = (event: KeyboardEvent) => {
      event.preventDefault();

      const key = getKey(event.key);
      if (key) {
        onChange(key);
      }

      setCapturing(false);
    };

    document.addEventListener('keydown', handleKeyDown);
    return () => document.removeEventListener('keydown', handleKeyDown);
  }, [capturing, onChange]);

  return (
    <button
      className={`${styles.capture} ${capturing ? styles.capturing : ''}`}
      onClick={() => setCapturing(true)}
      onBlur={() => setCapturing(false)}
    >
      {capturing ? 'Press a key…' : getKeyLabel(value)}
    </button>
  );
}
