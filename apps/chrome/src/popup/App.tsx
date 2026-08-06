import { Logo } from '@/shared/components/ui/logo';
import { SettingsIcon } from '@/shared/components/ui/settings-icon';

import styles from './App.module.css';

export default function App() {
  const handleClickSettingsButton = () => {
    chrome.runtime.openOptionsPage();
  };

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

      <p className={styles.tagline}>
        Copy any element on a page as text, HTML, Markdown, or an image.
      </p>
    </div>
  );
}
