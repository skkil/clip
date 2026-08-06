import { emitTransition } from '@/content/util/events';
import { DEFAULT_PREFERENCES, type Preferences } from '@/shared/preferences';

import CopyState from './copy';
import MenuState from './menu';
import MultiState from './multi';
import NormalState from './normal';
import ScreenshotState from './screenshot';
import { type IApp, type State } from './state';
import { StateType } from './types';

interface Context {
  state: StateType;
  mouse: {
    position: {
      x: number;
      y: number;
    };
  };
  elements: {
    hovered: HTMLElement | null;
  };
  preferences: Preferences;
}

export class App implements IApp {
  #context: Context = {
    state: StateType.NORMAL,
    mouse: {
      position: {
        x: 0,
        y: 0,
      },
    },
    elements: {
      hovered: null,
    },
    preferences: structuredClone(DEFAULT_PREFERENCES),
  };

  #states: Record<StateType, State> = {
    [StateType.NORMAL]: new NormalState(this),
    [StateType.COPY_MODE]: new CopyState(this),
    [StateType.MENU]: new MenuState(this),
    [StateType.MULTI]: new MultiState(this),
    [StateType.SCREENSHOT]: new ScreenshotState(this),
  };

  get hoveredElement(): HTMLElement | null {
    return this.#context.elements.hovered;
  }

  set hoveredElement(element: HTMLElement | null) {
    this.#context.elements.hovered = element;
  }

  get mousePosition(): { x: number; y: number } {
    return this.#context.mouse.position;
  }

  get preferences(): Preferences {
    return this.#context.preferences;
  }

  applyPreferences(preferences: Preferences): void {
    this.#context.preferences = preferences;
  }

  reset(): void {
    this.transition(StateType.NORMAL);
    this.hoveredElement = null;
  }

  transition(state: StateType): void {
    const current = this.#context.state;
    this.#states[current].onExit();

    this.#context.state = state;
    this.#states[state].onEnter();

    emitTransition({ from: current, to: state });
  }

  handleKeyDown(e: KeyboardEvent): void {
    this.#states[this.#context.state].handleKeyDown(e);
  }

  handleKeyUp(e: KeyboardEvent): void {
    this.#states[this.#context.state].handleKeyUp(e);
  }

  handleMouseOver(e: MouseEvent): void {
    this.hoveredElement = e.target as HTMLElement;
    this.#states[this.#context.state].handleMouseOver(e);
  }

  handleMouseOut(e: MouseEvent): void {
    this.#states[this.#context.state].handleMouseOut(e);

    const target = e.target as HTMLElement;
    if (this.hoveredElement === target) {
      this.hoveredElement = null;
    }
  }

  handleMouseDown(e: MouseEvent): void {
    this.#states[this.#context.state].handleMouseDown(e);
  }

  handleMouseMove(e: MouseEvent): void {
    this.#context.mouse.position.x = e.clientX;
    this.#context.mouse.position.y = e.clientY;
    this.#states[this.#context.state].handleMouseMove(e);
  }

  handleMouseUp(e: MouseEvent): void {
    this.#states[this.#context.state].handleMouseUp(e);
  }

  handleClick(e: MouseEvent): void {
    this.#states[this.#context.state].handleClick(e);
  }
}

const app = new App();

export default app;
