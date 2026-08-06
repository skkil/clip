import { copyElementAsTextToClipboard } from '@/content/util/clipboard';
import { setCursor } from '@/content/util/dom/cursor';
import { getKey } from '@/shared/utils/key';

import { State } from './state';
import { StateType } from './types';

export default class CopyState extends State {
  onEnter(): void {
    setCursor('copy');

    const hoveredElement = this.app.hoveredElement;
    if (hoveredElement) {
      hoveredElement.classList.add('clip-hovered-element');
    }
  }

  onExit(): void {
    const hoveredElement = this.app.hoveredElement;
    if (hoveredElement) {
      hoveredElement.classList.remove('clip-hovered-element');
    }
  }

  handleKeyDown(e: KeyboardEvent): void {
    if (e.key === 'Shift') {
      this.app.transition(StateType.MENU);
    } else if (e.key.toLowerCase() === 's') {
      this.app.transition(StateType.SCREENSHOT);
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
      hoveredElement.classList.add('clip-hovered-element');
    }
  }

  handleMouseOut(e: MouseEvent): void {
    const hoveredElement = this.app.hoveredElement;
    if (hoveredElement && e.target === hoveredElement) {
      hoveredElement.classList.remove('clip-hovered-element');
    }
  }

  handleClick(e: MouseEvent): void {
    e.preventDefault();
    e.stopPropagation();

    copyElementAsTextToClipboard(e.target as HTMLElement);
  }
}
