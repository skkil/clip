import type { CSSProperties, ReactNode } from 'react';

interface FixedContainerProps {
  id?: string;
  className?: string;
  style?: CSSProperties;
  children?: ReactNode;
}

export function FixedContainer({
  id,
  className,
  style,
  children,
}: FixedContainerProps) {
  return (
    <div id={id} className={className} style={{ position: 'fixed', ...style }}>
      {children}
    </div>
  );
}
