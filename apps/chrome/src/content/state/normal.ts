import { getKey } from "../../shared/utils/key";
import { State } from "./state";
import { StateType } from "./types";

export default class NormalState extends State {
  onEnter(): void {
    document.body.style.cursor = "default";
  }

  onExit(): void {}

  handleKeyDown(e: KeyboardEvent): void {
    const key = getKey(e.key);

    if (key === this.app.preferences.keys.leader) {
      this.app.transition(StateType.COPY_MODE);
    }
  }

  handleKeyUp(): void {}

  handleMouseOver(): void {}

  handleMouseOut(): void {}

  handleClick(): void {}
}
