import { type IApp, type State } from "./state";
import { StateType } from "./types";
import NormalState from "./normal";
import CopyState from "./copy";
import MenuState from "./menu";

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
  };

  #states: Record<StateType, State> = {
    [StateType.NORMAL]: new NormalState(this),
    [StateType.COPY_MODE]: new CopyState(this),
    [StateType.MENU]: new MenuState(this),
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

  transition(state: StateType): void {
    const current = this.#context.state;
    this.#states[current].onExit();

    this.#context.state = state;
    this.#states[state].onEnter();
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

  handleMouseMove(e: MouseEvent): void {
    this.#context.mouse.position.x = e.clientX;
    this.#context.mouse.position.y = e.clientY;
  }

  handleClick(e: MouseEvent): void {
    this.#states[this.#context.state].handleClick(e);
  }
}

export default new App();
