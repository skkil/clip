import { domToBlob } from 'modern-screenshot';
import TurndownService from 'turndown';

import { emitCopy } from './events';

const turndown = new TurndownService();

export function copyTextToClipboard(text: string): void {
  if (navigator.clipboard && navigator.clipboard.writeText) {
    navigator.clipboard
      .writeText(text)
      .then(() => {
        emitCopy({ format: 'text' });
      })
      .catch((err) => {
        console.error('Failed to copy text:', err);
      });
  }
}

export function copyElementAsTextToClipboard(element: HTMLElement): void {
  const textContent = element.innerText || element.textContent || '';
  if (textContent.trim()) {
    copyTextToClipboard(textContent.trim());
  }
}

export function copyElementAsHtmlToClipboard(element: HTMLElement): void {
  copyTextToClipboard(element.outerHTML);
}

export function copyElementAsMarkdownToClipboard(element: HTMLElement): void {
  copyTextToClipboard(turndown.turndown(element.outerHTML));
}

export async function copyElementAsImageToClipboard(
  element: HTMLElement,
): Promise<void> {
  const blob = await domToBlob(element, { type: 'image/png' });
  if (blob) {
    await navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })]);
    emitCopy({ format: 'image' });
  }
}

export function saveElementAsTextToFile(element: HTMLElement): void {
  const textContent = element.innerText || element.textContent || '';
  if (!textContent.trim()) {
    return;
  }

  const blob = new Blob([textContent.trim()], { type: 'text/plain' });
  const url = URL.createObjectURL(blob);

  const link = document.createElement('a');
  link.href = url;
  link.download = 'clip.txt';
  link.click();

  URL.revokeObjectURL(url);
}
