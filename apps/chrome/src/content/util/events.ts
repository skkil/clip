import type { StateType } from '@/content/state/types';

export const ClipEvent = {
  TRANSITION: 'clip:transition',
  COPY: 'clip:copy',
} as const;

export interface TransitionEventDetail {
  from: StateType;
  to: StateType;
}

export interface CopyEventDetail {
  format: 'text' | 'image';
}

function emit<T>(type: string, detail: T): void {
  if (window.location.protocol !== 'chrome-extension:') {
    return;
  }

  document.dispatchEvent(new CustomEvent<T>(type, { detail }));
}

export function emitTransition(detail: TransitionEventDetail): void {
  emit(ClipEvent.TRANSITION, detail);
}

export function emitCopy(detail: CopyEventDetail): void {
  emit(ClipEvent.COPY, detail);
}
