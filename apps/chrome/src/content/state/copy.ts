import { State } from "./state";
import { StateType } from "./types";

export default class CopyState extends State {
  onEnter(): void {
    document.body.style.cursor = "copy";
  }

  onExit(): void {
    const hoveredElement = this.app.hoveredElement;
    if (hoveredElement) {
      hoveredElement.style.outline = "";
      this.app.hoveredElement = null;
    }
  }

  handleKeyDown(): void {}

  handleKeyUp(e: KeyboardEvent): void {
    if (e.key === "Alt") {
      this.app.transition(StateType.NORMAL);
    }
  }

  handleMouseOver(e: MouseEvent): void {
    const hoveredElement = this.app.hoveredElement;
    const target = e.target as HTMLElement;

    if (hoveredElement && hoveredElement !== target) {
      hoveredElement.style.outline = "";
    }

    this.app.hoveredElement = target;
    target.style.outline = "1px solid red";
  }

  handleMouseOut(e: MouseEvent): void {
    const hoveredElement = this.app.hoveredElement;
    if (hoveredElement && e.target === hoveredElement) {
      hoveredElement.style.outline = "";
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
