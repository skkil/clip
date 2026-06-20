import { getKey } from "../../shared/utils/key";
import { State } from "./state";
import { StateType } from "./types";

export default class CopyState extends State {
  onEnter(): void {
    document.body.style.cursor = "copy";

    const hoveredElement = this.app.hoveredElement;
    if (hoveredElement) {
      hoveredElement.classList.add("clip-hovered-element");
    }
  }

  onExit(): void {
    const hoveredElement = this.app.hoveredElement;
    if (hoveredElement) {
      hoveredElement.classList.remove("clip-hovered-element");
    }
  }

  handleKeyDown(e: KeyboardEvent): void {
    if (e.key === "Shift") {
      this.app.transition(StateType.MENU);
    }
  }

  handleKeyUp(e: KeyboardEvent): void {
    if (this.app.preferences.keys.leader === getKey(e.key)) {
      this.app.transition(StateType.NORMAL);
    }
  }

  handleMouseOver(): void {
    const hoveredElement = this.app.hoveredElement;
    if (hoveredElement) {
      hoveredElement.classList.add("clip-hovered-element");
    }
  }

  handleMouseOut(e: MouseEvent): void {
    const hoveredElement = this.app.hoveredElement;
    if (hoveredElement && e.target === hoveredElement) {
      hoveredElement.classList.remove("clip-hovered-element");
    }
  }

  handleClick(e: MouseEvent): void {
    e.preventDefault();
    e.stopPropagation();

    const element = e.target as HTMLElement;
    const textContent = element.innerText || element.textContent || "";

    if (textContent.trim()) {
      navigator.clipboard.writeText(textContent.trim()).catch((err) => {
        console.error("Failed to copy text:", err);
      });
    }
  }
}
