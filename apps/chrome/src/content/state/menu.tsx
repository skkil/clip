import Menu from '@/content/components/menu/Menu';
import css from '@/content/components/menu/Menu.module.css?inline';
import {
  copyElementAsHtmlToClipboard,
  copyElementAsImageToClipboard,
  copyElementAsMarkdownToClipboard,
  copyElementAsTextToClipboard,
  saveElementAsTextToFile,
} from '@/content/util/clipboard';
import { type Mount, mount } from '@/content/util/dom/mount';
import { Key } from '@/shared/types/key';
import { getKey } from '@/shared/utils/key';

import { State } from './state';
import { StateType } from './types';

interface MenuItem {
  key: string;
  label: string;
  onClick: () => void;
}

export default class MenuState extends State {
  #selected: HTMLElement | null = null;
  #mount: Mount | null = null;

  #items: MenuItem[] = [
    {
      key: 'T',
      label: '[T]ext',
      onClick: () => {
        copyElementAsTextToClipboard(this.#selected!);
      },
    },
    {
      key: 'H',
      label: '[H]TML',
      onClick: () => {
        copyElementAsHtmlToClipboard(this.#selected!);
      },
    },
    {
      key: 'I',
      label: '[I]mage',
      onClick: () => {
        void copyElementAsImageToClipboard(this.#selected!);
      },
    },
    {
      key: 'M',
      label: '[M]arkdown',
      onClick: () => {
        copyElementAsMarkdownToClipboard(this.#selected!);
      },
    },
    {
      key: 'S',
      label: '[S]ave to file',
      onClick: () => {
        saveElementAsTextToFile(this.#selected!);
      },
    },
  ];

  onEnter(): void {
    this.#selected = this.app.hoveredElement;
    if (this.#selected) {
      this.#selected.classList.add('clip-selected-element');
    }

    this.#mount = mount(
      <Menu
        position={this.app.mousePosition}
        items={this.#items.map(({ key, label }) => ({ key, label }))}
        onItemClick={(key: string) => {
          const item = this.#items.find((i) => i.key === key);
          item?.onClick();
          this.app.transition(StateType.NORMAL);
        }}
        onCenterClick={() => {
          this.app.transition(StateType.MULTI);
        }}
      />,
      css,
    );
  }

  onExit(): void {
    if (this.#selected) {
      this.#selected.classList.remove('clip-selected-element');
    }

    this.#mount?.unmount();
    this.#mount = null;
  }

  handleKeyDown(e: KeyboardEvent): void {
    if (e.key === 'Escape') {
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
    const key = getKey(e.key);

    if (key === this.app.preferences.keys.leader) {
      this.app.transition(StateType.NORMAL);
    } else if (key === Key.SHIFT) {
      this.app.transition(StateType.COPY_MODE);
    }
  }

  handleMouseOut(): void {}

  handleClick(): void {
    this.app.transition(StateType.NORMAL);
  }
}
