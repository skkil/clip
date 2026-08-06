import { useState } from 'react';

import styles from './Options.module.css';
import { Sidebar } from './components/sidebar/Sidebar';
import type { SectionId } from './sections';
import { General } from './sections/general/General';

export default function Options() {
  const [activeSection, setActiveSection] = useState<SectionId>('general');

  return (
    <div className={styles.layout}>
      <Sidebar
        activeSection={activeSection}
        onSelectSection={setActiveSection}
      />

      <main className={styles.main}>
        <div className={styles.content}>
          {activeSection === 'general' && <General />}
        </div>
      </main>
    </div>
  );
}
