import type { CSSProperties } from 'react';

import { FixedContainer } from '@/shared/components/ui/container';

import styles from './Menu.module.css';

export interface MenuItemConfig {
  key: string;
  label: string;
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
      <div className={styles.center} onClick={onCenterClick}>
        {items.map((item, index) => {
          const angle = (index * (360 / items.length) - 90) * (Math.PI / 180);
          const x = Math.cos(angle) * 75;
          const y = Math.sin(angle) * 75;

          return (
            <div
              key={item.key}
              className={styles.item}
              style={{ '--x': `${x}px`, '--y': `${y}px` } as CSSProperties}
              onClick={(e) => {
                e.stopPropagation();
                onItemClick(item.key);
              }}
            >
              {item.label}
            </div>
          );
        })}
      </div>
    </FixedContainer>
  );
}
