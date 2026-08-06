import { SECTIONS, type SectionId } from '@/options/sections';
import { Logo } from '@/shared/components/ui/logo';
import { SettingsIcon } from '@/shared/components/ui/settings-icon';
import { ShieldIcon } from '@/shared/components/ui/shield-icon';

import styles from './Sidebar.module.css';

const PRIVACY_POLICY_URL =
  'https://github.com/skkil/clip/blob/main/apps/chrome/PRIVACY.md';

interface SidebarProps {
  activeSection: SectionId;
  onSelectSection: (section: SectionId) => void;
}

export function Sidebar({ activeSection, onSelectSection }: SidebarProps) {
  const version = chrome.runtime.getManifest().version;

  return (
    <aside className={styles.sidebar}>
      <div className={styles.brand}>
        <span className={styles.mark}>
          <Logo size={20} />
        </span>

        <h1 className={styles.wordmark}>
          CLIP
          <span className={styles.version}>v{version}</span>
        </h1>
      </div>

      <nav className={styles.nav}>
        {SECTIONS.map((section) => (
          <button
            key={section.id}
            className={styles.item}
            aria-current={section.id === activeSection ? 'page' : undefined}
            onClick={() => onSelectSection(section.id)}
          >
            <span className={styles.icon}>
              <SettingsIcon size={15} />
            </span>
            {section.label}
          </button>
        ))}
      </nav>

      <div className={styles.footer}>
        <a
          className={styles.privacy}
          href={PRIVACY_POLICY_URL}
          target="_blank"
          rel="noreferrer"
        >
          <ShieldIcon />
          Privacy Policy
        </a>
      </div>
    </aside>
  );
}
