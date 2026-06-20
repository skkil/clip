import type { Preferences } from "../../shared/preferences";
import { StateType } from "./types";

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
  abstract handleKeyDown(e: KeyboardEvent): void;
  abstract handleKeyUp(e: KeyboardEvent): void;
  abstract handleMouseOver(e: MouseEvent): void;
  abstract handleMouseOut(e: MouseEvent): void;
  abstract handleClick(e: MouseEvent): void;
}
