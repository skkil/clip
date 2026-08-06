import { restoreCursor } from '@/content/util/dom/cursor';
import { getKey } from '@/shared/utils/key';

import { State } from './state';
import { StateType } from './types';

export default class NormalState extends State {
  onEnter(): void {
    restoreCursor();
  }

  onExit(): void {}

  handleKeyDown(e: KeyboardEvent): void {
    const key = getKey(e.key);

    if (key === this.app.preferences.keys.leader) {
      this.app.transition(StateType.COPY_MODE);
    }
  }
}
