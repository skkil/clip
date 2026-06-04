import {
  copyElementAsTextToClipboard,
  copyElementAsHtmlToClipboard,
  copyElementAsImageToClipboard,
} from "../util/clipboard";
import { State } from "./state";
import { StateType } from "./types";

interface MenuItem {
  key: string;
  label: string;
  onClick: () => void;
}

export default class MenuState extends State {
  #selected: HTMLElement | null = null;

  #items: MenuItem[] = [
    {
      key: "T",
      label: "[T]ext",
      onClick: () => {
        copyElementAsTextToClipboard(this.#selected!);
      },
    },
    {
      key: "H",
      label: "[H]TML",
      onClick: () => {
        copyElementAsHtmlToClipboard(this.#selected!);
      },
    },
    {
      key: "I",
      label: "[I]mage",
      onClick: () => {
        copyElementAsImageToClipboard(this.#selected!);
      },
    },
  ];

  onEnter(): void {
    this.#constructMenu();

    this.#selected = this.app.hoveredElement;
    if (this.#selected) {
      this.#selected.classList.add("clip-selected-element");
    }
  }

  onExit(): void {
    if (this.#selected) {
      this.#selected.classList.remove("clip-selected-element");
    }

    this.#destroyMenu();
  }

  handleKeyDown(e: KeyboardEvent): void {
    if (e.key === "Escape") {
      e.preventDefault();
      this.app.transition(StateType.COPY_MODE);
    }

    const item = this.#items.find((i) => i.key === e.key);
    if (item) {
      e.preventDefault();
      item.onClick();
      this.app.transition(StateType.NORMAL);
    }
  }

  handleKeyUp(e: KeyboardEvent): void {
    if (e.key === "Alt") {
      this.app.transition(StateType.NORMAL);
    } else if (e.key === "Shift") {
      this.app.transition(StateType.COPY_MODE);
    }
  }

  handleMouseOver(): void {}

  handleMouseOut(): void {}

  handleClick(): void {
    this.app.transition(StateType.NORMAL);
  }

  #constructMenu() {
    const position = this.app.mousePosition;

    const container = document.createElement("div");
    container.id = "clip-menu-container";
    container.style.left = `${position.x}px`;
    container.style.top = `${position.y}px`;

    const center = document.createElement("div");
    center.id = "clip-menu-center";

    this.#items.forEach((item, index) => {
      const e = document.createElement("div");
      e.className = "clip-menu-item";
      e.textContent = item.label;

      const angle = (index * (360 / this.#items.length) - 90) * (Math.PI / 180);

      const x = Math.cos(angle) * 75;
      const y = Math.sin(angle) * 75;

      e.style.setProperty("--x", `${x}px`);
      e.style.setProperty("--y", `${y}px`);

      e.addEventListener("click", (event) => {
        event.stopPropagation();
        item.onClick();
        this.app.transition(StateType.NORMAL);
      });

      center.appendChild(e);
    });

    container.appendChild(center);

    document.body.appendChild(container);
  }

  #destroyMenu() {
    document.getElementById("clip-menu-container")?.remove();
  }
}
