import { type State } from "./state";
import { StateType } from "./types";
import NormalState from "./normal";
import CopyState from "./copy";

interface Context {
  state: StateType;
  elements: {
    hovered: HTMLElement | null;
  };
}

export class App {
  #context: Context = {
    state: StateType.NORMAL,
    elements: {
      hovered: null,
    },
  };

  #states: Record<StateType, State> = {
    [StateType.NORMAL]: new NormalState(this),
    [StateType.COPY_MODE]: new CopyState(this),
  };

  get hoveredElement(): HTMLElement | null {
    return this.#context.elements.hovered;
  }

  set hoveredElement(element: HTMLElement | null) {
    this.#context.elements.hovered = element;
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
    this.#states[this.#context.state].handleMouseOver(e);
  }

  handleMouseOut(e: MouseEvent): void {
    this.#states[this.#context.state].handleMouseOut(e);
  }

  handleClick(e: MouseEvent): void {
    this.#states[this.#context.state].handleClick(e);
  }
}

export default new App();
