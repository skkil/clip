import { State } from "./state";
import { StateType } from "./types";

export default class NormalState extends State {
  onEnter(): void {
    document.body.style.cursor = "default";
  }

  onExit(): void {}

  handleKeyDown(e: KeyboardEvent): void {
    if (e.key === "Alt") {
      this.app.transition(StateType.COPY_MODE);
    }
  }

  handleKeyUp(): void {}

  handleMouseOver(): void {}

  handleMouseOut(): void {}

  handleClick(): void {}
}
