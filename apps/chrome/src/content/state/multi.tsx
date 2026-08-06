import SelectionBadges, {
  type SelectionBadge,
} from '@/content/components/selection/SelectionBadges';
import css from '@/content/components/selection/SelectionBadges.module.css?inline';
import { copyTextToClipboard } from '@/content/util/clipboard';
import { type Mount, mount } from '@/content/util/dom/mount';
import { Key } from '@/shared/types/key';
import { getKey } from '@/shared/utils/key';

import { State } from './state';
import { StateType } from './types';

const BADGE_MARGIN = 12;

export default class MultiState extends State {
  #elements: HTMLElement[] = [];
  #mount: Mount | null = null;

  #render = (): void => {
    const badges: SelectionBadge[] = this.#elements.map((element, index) => {
      const rect = element.getBoundingClientRect();

      return {
        order: index + 1,
        x: Math.max(rect.left, BADGE_MARGIN),
        y: Math.max(rect.top, BADGE_MARGIN),
      };
    });

    this.#mount?.render(<SelectionBadges badges={badges} />);
  };

  onEnter(): void {
    this.#mount = mount(<SelectionBadges badges={[]} />, css);

    window.addEventListener('scroll', this.#render, true);
    window.addEventListener('resize', this.#render);

    const hoveredElement = this.app.hoveredElement;
    if (hoveredElement) {
      hoveredElement.classList.add('clip-hovered-element');
      this.#elements.push(hoveredElement);
    }

    this.#render();
  }

  onExit(): void {
    window.removeEventListener('scroll', this.#render, true);
    window.removeEventListener('resize', this.#render);

    for (const element of this.#elements) {
      element.classList.remove('clip-hovered-element');
    }
    this.#elements = [];

    this.#mount?.unmount();
    this.#mount = null;
  }

  handleKeyDown(e: KeyboardEvent): void {
    const key = getKey(e.key);

    if (key === Key.ESCAPE) {
      this.app.transition(StateType.NORMAL);
    } else if (key === Key.ENTER) {
      const combinedText = this.#elements
        .map((element) => element.innerText || element.textContent || '')
        .filter((text) => text.trim())
        .join('\n\n');

      const count = this.#elements.length;
      void copyTextToClipboard(
        combinedText.trim(),
        `Copied ${count} element${count === 1 ? '' : 's'}`,
      );

      this.app.transition(StateType.NORMAL);
    }
  }

  handleMouseOver(): void {
    const hoveredElement = this.app.hoveredElement;
    if (hoveredElement) {
      hoveredElement.classList.add('clip-hovered-element');
    }
  }

  handleMouseOut(): void {
    const hoveredElement = this.app.hoveredElement;
    if (hoveredElement && !this.#elements.includes(hoveredElement)) {
      hoveredElement.classList.remove('clip-hovered-element');
    }
  }

  handleClick(e: MouseEvent): void {
    e.preventDefault();
    e.stopPropagation();

    const element = e.target as HTMLElement;
    if (this.#elements.includes(element)) {
      element.classList.remove('clip-hovered-element');
      const index = this.#elements.indexOf(element);
      this.#elements.splice(index, 1);
    } else {
      element.classList.add('clip-hovered-element');
      this.#elements.push(element);
    }

    this.#render();
  }
}
