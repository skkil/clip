import { State } from "./state";
import { StateType } from "./types";

export default class CopyState extends State {
  onEnter(): void {
    document.body.style.cursor = "copy";

    const hoveredElement = this.app.hoveredElement;
    if (hoveredElement) {
      hoveredElement.classList.add("sync-hovered-element");
    }
  }

  onExit(): void {
    const hoveredElement = this.app.hoveredElement;
    if (hoveredElement) {
      hoveredElement.classList.remove("sync-hovered-element");
    }
  }

  handleKeyDown(): void {}

  handleKeyUp(e: KeyboardEvent): void {
    if (e.key === "Alt") {
      this.app.transition(StateType.NORMAL);
    }
  }

  handleMouseOver(): void {
    const hoveredElement = this.app.hoveredElement;
    if (hoveredElement) {
      hoveredElement.classList.add("sync-hovered-element");
    }
  }

  handleMouseOut(e: MouseEvent): void {
    const hoveredElement = this.app.hoveredElement;
    if (hoveredElement && e.target === hoveredElement) {
      hoveredElement.classList.remove("sync-hovered-element");
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
