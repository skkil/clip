import { useEffect, useState } from 'react';

import { StateType } from '@/content/state/types';
import { ClipEvent, type TransitionEventDetail } from '@/content/util/events';
import { Kbd } from '@/shared/components/ui/kbd';
import { Logo } from '@/shared/components/ui/logo';
import { DEFAULT_PREFERENCES, getPreferences } from '@/shared/preferences';
import { getKeyLabel } from '@/shared/utils/key';

import styles from './Welcome.module.css';

function Check({ done }: { done: boolean }) {
  return (
    <span className={`${styles.check} ${done ? styles.checkDone : ''}`}>
      {done ? (
        <svg viewBox="0 0 16 16" width="12" height="12" aria-hidden="true">
          <path
            d="M3.5 8.5l3 3 6-7"
            fill="none"
            stroke="currentColor"
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
          />
        </svg>
      ) : null}
    </span>
  );
}

export default function Welcome() {
  const [leaderKey, setLeaderKey] = useState(DEFAULT_PREFERENCES.keys.leader);
  const [entered, setEntered] = useState(false);
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    getPreferences().then((preferences) => {
      setLeaderKey(preferences.keys.leader);
    });
  }, []);

  useEffect(() => {
    const handleTransition = (event: Event) => {
      const { detail } = event as CustomEvent<TransitionEventDetail>;
      if (detail.to === StateType.COPY_MODE) {
        setEntered(true);
      }
    };

    const handleCopy = () => {
      setCopied(true);
    };

    document.addEventListener(ClipEvent.TRANSITION, handleTransition);
    document.addEventListener(ClipEvent.COPY, handleCopy);

    return () => {
      document.removeEventListener(ClipEvent.TRANSITION, handleTransition);
      document.removeEventListener(ClipEvent.COPY, handleCopy);
    };
  }, []);

  const leader = getKeyLabel(leaderKey);
  const done = entered && copied;

  const handleClickSettings = () => {
    chrome.runtime.openOptionsPage();
  };

  return (
    <div className={styles.page}>
      <main className={styles.main}>
        <header className={styles.header}>
          <span className={styles.mark}>
            <Logo size={26} />
          </span>
          <div>
            <h1 className={styles.title}>CLIP is ready</h1>
            <p className={styles.subtitle}>
              One gesture copies anything on any page. Try it once below — it
              takes five seconds.
            </p>
          </div>
        </header>

        <section
          className={`${styles.practice} ${done ? styles.practiceDone : ''}`}
        >
          <ol className={styles.steps}>
            <li className={styles.step}>
              <Check done={entered} />
              <div className={styles.stepBody}>
                <p className={styles.stepTitle}>
                  Hold <Kbd>{leader}</Kbd> and move your cursor over the text
                  below
                </p>
                <p className={styles.stepHint}>
                  Whatever you hover gets outlined. That outline is what you are
                  about to copy.
                </p>
              </div>
            </li>

            <li className={`${styles.step} ${entered ? '' : styles.stepIdle}`}>
              <Check done={copied} />
              <div className={styles.stepBody}>
                <p className={styles.stepTitle}>
                  Still holding <Kbd>{leader}</Kbd>, click it
                </p>
                <p className={styles.stepHint}>
                  The text lands on your clipboard. Release the key to go back
                  to browsing normally.
                </p>
              </div>
            </li>
          </ol>

          <div className={styles.sandbox}>
            <p className={styles.sample}>
              Nothing here is special. CLIP treats this paragraph exactly the
              way it treats a paragraph on any other site — hold the key, click,
              and it is yours.
            </p>
          </div>

          <p className={styles.result} role="status">
            {done
              ? 'Copied. Paste it anywhere to check — that is the whole feature.'
              : 'Waiting for you to try it…'}
          </p>
        </section>

        <section className={styles.more}>
          <h2 className={styles.moreTitle}>Once that feels natural</h2>
          <p className={styles.moreSubtitle}>
            Every one of these starts the same way: hold <Kbd>{leader}</Kbd>.
          </p>

          <ul className={styles.cards}>
            <li className={styles.card}>
              <div className={styles.cardKeys}>
                <Kbd>{leader}</Kbd>
                <span className={styles.plus}>+</span>
                <Kbd>Shift</Kbd>
              </div>
              <h3 className={styles.cardTitle}>Pick a format</h3>
              <p className={styles.cardBody}>
                Opens a menu at your cursor: plain text, HTML, Markdown, an
                image of the element, or save it straight to a file.
              </p>
            </li>

            <li className={styles.card}>
              <div className={styles.cardKeys}>
                <Kbd>{leader}</Kbd>
                <span className={styles.plus}>+</span>
                <Kbd>S</Kbd>
              </div>
              <h3 className={styles.cardTitle}>Capture a region</h3>
              <p className={styles.cardBody}>
                Drag a box over any part of the page, press <Kbd>Enter</Kbd> to
                copy it as an image, or <Kbd>Esc</Kbd> to back out.
              </p>
            </li>

            <li className={styles.card}>
              <div className={styles.cardKeys}>
                <Kbd>{leader}</Kbd>
                <span className={styles.plus}>+</span>
                <Kbd>Shift</Kbd>
                <span className={styles.plus}>→</span>
                <span className={styles.cardKeysNote}>centre</span>
              </div>
              <h3 className={styles.cardTitle}>Collect several at once</h3>
              <p className={styles.cardBody}>
                Click the middle of the format menu, then click as many elements
                as you like. <Kbd>Enter</Kbd> copies them together.
              </p>
            </li>
          </ul>
        </section>

        <footer className={styles.footer}>
          <p className={styles.footerText}>
            <Kbd>{leader}</Kbd> not comfortable? Any modifier key works.
          </p>
          <button
            className={styles.settingsButton}
            onClick={handleClickSettings}
          >
            Change the leader key
          </button>
        </footer>
      </main>
    </div>
  );
}
