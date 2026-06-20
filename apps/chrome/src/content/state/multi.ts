import { copyTextToClipboard } from "../util/clipboard";
import { State } from "./state";
import { StateType } from "./types";

export default class MultiState extends State {
  #elements: HTMLElement[] = [];

  onEnter(): void {
    const hoveredElement = this.app.hoveredElement;
    if (hoveredElement) {
      hoveredElement.classList.add("clip-hovered-element");
      this.#elements.push(hoveredElement);
    }
  }

  onExit(): void {
    for (const element of this.#elements) {
      element.classList.remove("clip-hovered-element");
    }
    this.#elements = [];
  }

  handleKeyDown(e: KeyboardEvent): void {
    if (e.key === "Escape") {
      this.app.transition(StateType.NORMAL);
    } else if (e.key === "Enter") {
      const combinedText = this.#elements
        .map((element) => element.innerText || element.textContent || "")
        .filter((text) => text.trim())
        .join("\n\n");

      copyTextToClipboard(combinedText.trim());

      this.app.transition(StateType.NORMAL);
    }
  }

  handleKeyUp(): void {}

  handleMouseOver(): void {
    const hoveredElement = this.app.hoveredElement;
    if (hoveredElement) {
      hoveredElement.classList.add("clip-hovered-element");
    }
  }

  handleMouseOut(): void {
    const hoveredElement = this.app.hoveredElement;
    if (hoveredElement && !this.#elements.includes(hoveredElement)) {
      hoveredElement.classList.remove("clip-hovered-element");
    }
  }

  handleClick(e: MouseEvent): void {
    e.preventDefault();
    e.stopPropagation();

    const element = e.target as HTMLElement;
    if (this.#elements.includes(element)) {
      element.classList.remove("clip-hovered-element");
      const index = this.#elements.indexOf(element);
      this.#elements.splice(index, 1);
    } else {
      element.classList.add("clip-hovered-element");
      this.#elements.push(element);
    }
  }
}
