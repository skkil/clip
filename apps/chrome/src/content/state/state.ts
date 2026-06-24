import type { Preferences } from '@/shared/preferences';

import { StateType } from './types';

export interface IApp {
  hoveredElement: HTMLElement | null;
  mousePosition: { x: number; y: number };
  preferences: Preferences;
  transition(state: StateType): void;
}

export abstract class State {
  constructor(protected app: IApp) {}

  abstract onEnter(): void;
  abstract onExit(): void;

  handleKeyDown(_e: KeyboardEvent): void {}
  handleKeyUp(_e: KeyboardEvent): void {}

  handleClick(_e: MouseEvent): void {}

  handleMouseOver(_e: MouseEvent): void {}
  handleMouseOut(_e: MouseEvent): void {}
  handleMouseDown(_e: MouseEvent): void {}
  handleMouseMove(_e: MouseEvent): void {}
  handleMouseUp(_e: MouseEvent): void {}
}
