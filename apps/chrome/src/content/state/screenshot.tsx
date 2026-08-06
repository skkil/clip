import { createRef } from 'react';

import Screenshot, {
  type ScreenshotHandle,
} from '@/content/components/screenshot/Screenshot';
import css from '@/content/components/screenshot/Screenshot.module.css?inline';
import { setCursor } from '@/content/util/dom/cursor';
import { type Mount, mount } from '@/content/util/dom/mount';
import { MessageType } from '@/shared/messages';

import { State } from './state';
import { StateType } from './types';

export default class ScreenshotState extends State {
  #mount: Mount | null = null;
  #ref = createRef<ScreenshotHandle>();

  #isSelecting = false;
  #startX = 0;
  #startY = 0;
  #selection: { x: number; y: number; width: number; height: number } | null =
    null;

  onEnter(): void {
    setCursor('crosshair');

    this.#mount = mount(<Screenshot ref={this.#ref} />, css);
  }

  onExit(): void {
    this.#mount?.unmount();
    this.#mount = null;
    this.#isSelecting = false;
    this.#selection = null;
  }

  handleKeyDown(e: KeyboardEvent): void {
    if (e.key === 'Escape') {
      this.app.transition(StateType.NORMAL);
    } else if (e.key === 'Enter' && this.#selection) {
      const { x, y, width, height } = this.#selection;
      this.#captureSelection(x, y, width, height);
    }
  }

  handleClick(e: MouseEvent): void {
    e.preventDefault();
    e.stopPropagation();
  }

  handleMouseDown(e: MouseEvent): void {
    e.preventDefault();

    this.#isSelecting = true;
    this.#startX = e.clientX;
    this.#startY = e.clientY;

    this.#ref.current?.startSelection();
    this.#updateSelection(e.clientX, e.clientY);
  }

  handleMouseMove(e: MouseEvent): void {
    if (!this.#isSelecting) {
      return;
    }

    this.#updateSelection(e.clientX, e.clientY);
  }

  handleMouseUp(e: MouseEvent): void {
    if (!this.#isSelecting) {
      return;
    }
    this.#isSelecting = false;

    const x = Math.min(e.clientX, this.#startX);
    const y = Math.min(e.clientY, this.#startY);
    const width = Math.abs(e.clientX - this.#startX);
    const height = Math.abs(e.clientY - this.#startY);

    if (width < 4 || height < 4) {
      this.#selection = null;
      this.#ref.current?.endSelection(false);
      return;
    }

    this.#selection = { x, y, width, height };
  }

  #updateSelection(currentX: number, currentY: number): void {
    const x = Math.min(currentX, this.#startX);
    const y = Math.min(currentY, this.#startY);
    const width = Math.abs(currentX - this.#startX);
    const height = Math.abs(currentY - this.#startY);

    this.#ref.current?.updateSelection(x, y, width, height);
  }

  async #captureSelection(
    x: number,
    y: number,
    width: number,
    height: number,
  ): Promise<void> {
    try {
      this.#mount?.unmount();
      this.#mount = null;
      await new Promise((resolve) => requestAnimationFrame(resolve));

      const dataUrl = await this.#captureVisibleTab();
      if (!dataUrl) {
        return;
      }

      const image = await this.#loadImage(dataUrl);
      const scale = image.naturalWidth / window.innerWidth;

      const canvas = document.createElement('canvas');
      canvas.width = width * scale;
      canvas.height = height * scale;

      const context = canvas.getContext('2d');
      if (!context) {
        return;
      }

      context.drawImage(
        image,
        x * scale,
        y * scale,
        width * scale,
        height * scale,
        0,
        0,
        width * scale,
        height * scale,
      );

      const croppedBlob = await new Promise<Blob | null>((resolve) => {
        canvas.toBlob(resolve, 'image/png');
      });

      if (croppedBlob) {
        await navigator.clipboard.write([
          new ClipboardItem({ 'image/png': croppedBlob }),
        ]);
      }
    } catch (err) {
      console.error('Failed to capture screenshot:', err);
    } finally {
      this.app.transition(StateType.NORMAL);
    }
  }

  #captureVisibleTab(): Promise<string | null> {
    return new Promise((resolve) => {
      chrome.runtime.sendMessage(
        { type: MessageType.CAPTURE_SELECTION },
        (response: { dataUrl?: string; error?: string } | undefined) => {
          if (chrome.runtime.lastError || !response?.dataUrl) {
            console.error(
              'Failed to capture visible tab:',
              chrome.runtime.lastError?.message ?? response?.error,
            );
            resolve(null);
            return;
          }

          resolve(response.dataUrl);
        },
      );
    });
  }

  #loadImage(src: string): Promise<HTMLImageElement> {
    return new Promise((resolve, reject) => {
      const image = new Image();
      image.onload = () => resolve(image);
      image.onerror = reject;
      image.src = src;
    });
  }
}
