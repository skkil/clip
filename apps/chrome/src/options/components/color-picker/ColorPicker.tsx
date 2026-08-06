import { useId } from 'react';

import styles from './ColorPicker.module.css';

const PRESETS = [
  { value: '#ff0000', label: 'Red' },
  { value: '#f97316', label: 'Orange' },
  { value: '#facc15', label: 'Yellow' },
  { value: '#22c55e', label: 'Green' },
  { value: '#3b82f6', label: 'Blue' },
  { value: '#a855f7', label: 'Purple' },
];

interface ColorPickerProps {
  value: string;
  label: string;
  onChange: (color: string) => void;
}

export function ColorPicker({ value, label, onChange }: ColorPickerProps) {
  const inputId = useId();
  const normalized = value.toLowerCase();

  return (
    <div className={styles.picker} role="group" aria-label={label}>
      {PRESETS.map((preset) => (
        <button
          key={preset.value}
          type="button"
          aria-label={preset.label}
          aria-pressed={normalized === preset.value}
          className={`${styles.swatch} ${normalized === preset.value ? styles.selected : ''}`}
          style={{ background: preset.value }}
          onClick={() => onChange(preset.value)}
        />
      ))}

      <label
        htmlFor={inputId}
        className={styles.custom}
        style={{ background: value }}
        title="Custom color"
      >
        <input
          id={inputId}
          type="color"
          className={styles.input}
          value={value}
          aria-label="Custom color"
          onChange={(e) => onChange(e.target.value)}
        />
      </label>
    </div>
  );
}
