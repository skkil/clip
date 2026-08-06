import type { ReactNode } from 'react';

import styles from './Setting.module.css';

export function SettingGroup({ children }: { children: ReactNode }) {
  return <div className={styles.group}>{children}</div>;
}

interface SettingProps {
  label: string;
  description?: string;
  children: ReactNode;
}

export function Setting({ label, description, children }: SettingProps) {
  return (
    <div className={styles.setting}>
      <div className={styles.text}>
        <div className={styles.label}>{label}</div>
        {description && <p className={styles.description}>{description}</p>}
      </div>

      <div className={styles.control}>{children}</div>
    </div>
  );
}
