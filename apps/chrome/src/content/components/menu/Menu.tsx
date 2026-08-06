import { CopyIcon, IconContext } from '@phosphor-icons/react';
import type { CSSProperties, ReactNode } from 'react';

import { FixedContainer } from '@/shared/components/ui/container';

import styles from './Menu.module.css';

const RADIUS = 72;

export interface MenuItemConfig {
  key: string;
  label: string;
  icon: ReactNode;
}

interface MenuProps {
  position: { x: number; y: number };
  items: MenuItemConfig[];
  onItemClick: (key: string) => void;
  onCenterClick: () => void;
}

export default function Menu({
  position,
  items,
  onItemClick,
  onCenterClick,
}: MenuProps) {
  return (
    <FixedContainer
      id="clip-menu-container"
      className={styles.container}
      style={{ left: `${position.x}px`, top: `${position.y}px` }}
    >
      <IconContext.Provider value={{ size: 20, weight: 'bold' }}>
        <div
          className={styles.center}
          aria-label="Select multiple elements"
          title="Select multiple elements"
          onClick={onCenterClick}
        >
          <CopyIcon size={18} />

          {items.map((item, index) => {
            const angle = (index * (360 / items.length) - 90) * (Math.PI / 180);
            const x = Math.cos(angle) * RADIUS;
            const y = Math.sin(angle) * RADIUS;

            return (
              <div
                key={item.key}
                className={styles.item}
                aria-label={item.label}
                title={item.label}
                style={
                  {
                    '--x': `${x}px`,
                    '--y': `${y}px`,
                    '--index': index,
                  } as CSSProperties
                }
                onClick={(e) => {
                  e.stopPropagation();
                  onItemClick(item.key);
                }}
              >
                {item.icon}
              </div>
            );
          })}
        </div>
      </IconContext.Provider>
    </FixedContainer>
  );
}
