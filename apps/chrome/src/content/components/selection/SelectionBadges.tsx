import { FixedContainer } from '@/shared/components/ui/container';

import styles from './SelectionBadges.module.css';

export interface SelectionBadge {
  order: number;
  x: number;
  y: number;
}

interface SelectionBadgesProps {
  badges: SelectionBadge[];
}

export default function SelectionBadges({ badges }: SelectionBadgesProps) {
  return (
    <>
      {badges.map((badge) => (
        <FixedContainer
          key={badge.order}
          className={styles.badge}
          style={{ left: `${badge.x}px`, top: `${badge.y}px` }}
        >
          {badge.order}
        </FixedContainer>
      ))}
    </>
  );
}
