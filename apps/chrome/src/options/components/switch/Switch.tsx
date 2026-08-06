import styles from './Switch.module.css';

interface SwitchProps {
  checked: boolean;
  label: string;
  onChange: (checked: boolean) => void;
}

export function Switch({ checked, label, onChange }: SwitchProps) {
  return (
    <button
      type="button"
      role="switch"
      aria-checked={checked}
      aria-label={label}
      className={`${styles.track} ${checked ? styles.on : ''}`}
      onClick={() => onChange(!checked)}
    >
      <span className={styles.thumb} />
    </button>
  );
}
