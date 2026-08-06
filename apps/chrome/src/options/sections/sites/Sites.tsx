import { type FormEvent, useEffect, useState } from 'react';

import {
  DEFAULT_PREFERENCES,
  type Preferences,
  SiteMode,
  getPreferences,
  setPreferences,
} from '@/shared/preferences';
import {
  addSitePattern,
  parseSitePattern,
  removeSitePattern,
} from '@/shared/sites';

import styles from './Sites.module.css';

const MODES = [
  {
    value: SiteMode.BLOCKLIST,
    label: 'Run everywhere except',
    description: 'CLIP is active on every site you visit, apart from the list.',
  },
  {
    value: SiteMode.ALLOWLIST,
    label: 'Run only on',
    description: 'CLIP stays inactive everywhere until a site is on the list.',
  },
] as const;

export function Sites() {
  const [sites, setSites] = useState<Preferences['sites']>(
    DEFAULT_PREFERENCES.sites,
  );
  const [input, setInput] = useState('');
  const [error, setError] = useState('');

  useEffect(() => {
    getPreferences().then((preferences) => {
      setSites(preferences.sites);
    });
  }, []);

  const save = async (next: Preferences['sites']) => {
    setSites(next);
    await setPreferences({ sites: next });
  };

  const handleChangeMode = (mode: SiteMode) => {
    void save({ ...sites, mode });
  };

  const handleSubmit = (event: FormEvent) => {
    event.preventDefault();

    const pattern = parseSitePattern(input);
    if (!pattern) {
      setError('Enter a site like example.com');
      return;
    }

    setInput('');
    setError('');
    void save(addSitePattern(sites, pattern));
  };

  const handleRemove = (pattern: string) => {
    void save(removeSitePattern(sites, pattern));
  };

  const entries =
    sites.mode === SiteMode.ALLOWLIST ? sites.allowlist : sites.blocklist;

  const emptyMessage =
    sites.mode === SiteMode.ALLOWLIST
      ? 'No sites yet, so CLIP is inactive everywhere.'
      : 'No sites yet, so CLIP runs everywhere.';

  return (
    <section className={styles.section}>
      <div>
        <h2 className={styles.heading}>Sites</h2>
        <p className={styles.subheading}>
          Choose where CLIP is allowed to attach itself.
        </p>
      </div>

      <div className={styles.modes} role="radiogroup" aria-label="Site mode">
        {MODES.map((mode) => (
          <button
            key={mode.value}
            type="button"
            role="radio"
            aria-checked={sites.mode === mode.value}
            className={styles.mode}
            onClick={() => handleChangeMode(mode.value)}
          >
            <span className={styles.radio} aria-hidden="true" />
            <span className={styles.modeText}>
              <span className={styles.modeLabel}>{mode.label}</span>
              <span className={styles.modeDescription}>{mode.description}</span>
            </span>
          </button>
        ))}
      </div>

      <form className={styles.form} onSubmit={handleSubmit}>
        <input
          className={styles.input}
          value={input}
          placeholder="example.com"
          aria-label="Site to add"
          spellCheck={false}
          autoComplete="off"
          onChange={(event) => {
            setInput(event.target.value);
            setError('');
          }}
        />
        <button className={styles.add} type="submit">
          Add
        </button>
      </form>

      <div className={styles.error}>{error}</div>

      {entries.length === 0 ? (
        <p className={styles.empty}>{emptyMessage}</p>
      ) : (
        <ul className={styles.list}>
          {entries.map((pattern) => (
            <li key={pattern} className={styles.entry}>
              <span className={styles.pattern}>{pattern}</span>
              <button
                className={styles.remove}
                onClick={() => handleRemove(pattern)}
                aria-label={`Remove ${pattern}`}
              >
                Remove
              </button>
            </li>
          ))}
        </ul>
      )}

      <p className={styles.note}>
        A site also covers its subdomains, so example.com includes
        app.example.com.
      </p>
    </section>
  );
}
