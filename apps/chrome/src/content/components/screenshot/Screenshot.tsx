import { forwardRef, useImperativeHandle, useRef, useState } from 'react';

import { FixedContainer } from '@/shared/components/ui/container';

import styles from './Screenshot.module.css';

export interface ScreenshotHandle {
  startSelection(): void;
  updateSelection(x: number, y: number, width: number, height: number): void;
  endSelection(valid: boolean): void;
}

const Screenshot = forwardRef<ScreenshotHandle>((_props, ref) => {
  const [selecting, setSelecting] = useState(false);

  const topShadeRef = useRef<HTMLDivElement>(null);
  const bottomShadeRef = useRef<HTMLDivElement>(null);
  const leftShadeRef = useRef<HTMLDivElement>(null);
  const rightShadeRef = useRef<HTMLDivElement>(null);
  const selectionBoxRef = useRef<HTMLDivElement>(null);

  useImperativeHandle(ref, () => ({
    startSelection() {
      setSelecting(true);
    },
    updateSelection(x, y, width, height) {
      if (topShadeRef.current) {
        topShadeRef.current.style.left = '0';
        topShadeRef.current.style.top = '0';
        topShadeRef.current.style.width = '100%';
        topShadeRef.current.style.height = `${y}px`;
      }
      if (bottomShadeRef.current) {
        bottomShadeRef.current.style.left = '0';
        bottomShadeRef.current.style.top = `${y + height}px`;
        bottomShadeRef.current.style.width = '100%';
        bottomShadeRef.current.style.height = `calc(100% - ${y + height}px)`;
      }
      if (leftShadeRef.current) {
        leftShadeRef.current.style.left = '0';
        leftShadeRef.current.style.top = `${y}px`;
        leftShadeRef.current.style.width = `${x}px`;
        leftShadeRef.current.style.height = `${height}px`;
      }
      if (rightShadeRef.current) {
        rightShadeRef.current.style.left = `${x + width}px`;
        rightShadeRef.current.style.top = `${y}px`;
        rightShadeRef.current.style.width = `calc(100% - ${x + width}px)`;
        rightShadeRef.current.style.height = `${height}px`;
      }
      if (selectionBoxRef.current) {
        selectionBoxRef.current.style.left = `${x}px`;
        selectionBoxRef.current.style.top = `${y}px`;
        selectionBoxRef.current.style.width = `${width}px`;
        selectionBoxRef.current.style.height = `${height}px`;
      }
    },
    endSelection(valid) {
      setSelecting(valid);
    },
  }));

  return (
    <FixedContainer id="clip-screenshot-overlay" className={styles.overlay}>
      {!selecting && <div className={styles.shade} style={{ inset: 0 }} />}
      {selecting && (
        <>
          <div ref={topShadeRef} className={styles.shade} />
          <div ref={bottomShadeRef} className={styles.shade} />
          <div ref={leftShadeRef} className={styles.shade} />
          <div ref={rightShadeRef} className={styles.shade} />
          <div ref={selectionBoxRef} className={styles.selectionBox} />
        </>
      )}
    </FixedContainer>
  );
});

Screenshot.displayName = 'Screenshot';

export default Screenshot;
