import { domToBlob } from 'modern-screenshot';
import TurndownService from 'turndown';

import { emitCopy } from './events';
import { showToast } from './toast';

const turndown = new TurndownService();

function writeTextWithCommand(text: string): boolean {
  const textarea = document.createElement('textarea');
  textarea.value = text;
  textarea.setAttribute('readonly', '');
  textarea.style.position = 'fixed';
  textarea.style.top = '0';
  textarea.style.left = '-9999px';
  textarea.style.opacity = '0';

  document.body.appendChild(textarea);

  const selection = document.getSelection();
  const previousRange =
    selection && selection.rangeCount > 0 ? selection.getRangeAt(0) : null;
  const previousFocus = document.activeElement;

  textarea.focus();
  textarea.select();

  let copied = false;
  try {
    copied = document.execCommand('copy');
  } catch (err) {
    console.error('Failed to copy text:', err);
  }

  textarea.remove();

  if (selection && previousRange) {
    selection.removeAllRanges();
    selection.addRange(previousRange);
  }

  if (previousFocus instanceof HTMLElement) {
    previousFocus.focus();
  }

  return copied;
}

export async function copyTextToClipboard(
  text: string,
  label = 'Copied',
): Promise<void> {
  if (navigator.clipboard?.writeText) {
    try {
      await navigator.clipboard.writeText(text);
      emitCopy({ format: 'text' });
      showToast('success', label);
      return;
    } catch (err) {
      console.error('Failed to copy text:', err);
    }
  }

  if (writeTextWithCommand(text)) {
    emitCopy({ format: 'text' });
    showToast('success', label);
    return;
  }

  showToast('error', 'Could not copy');
}

export function copyElementAsTextToClipboard(element: HTMLElement): void {
  const textContent = element.innerText || element.textContent || '';
  if (!textContent.trim()) {
    showToast('info', 'Nothing to copy here');
    return;
  }

  void copyTextToClipboard(textContent.trim());
}

export function copyElementAsHtmlToClipboard(element: HTMLElement): void {
  void copyTextToClipboard(element.outerHTML, 'Copied as HTML');
}

export function copyElementAsMarkdownToClipboard(element: HTMLElement): void {
  void copyTextToClipboard(
    turndown.turndown(element.outerHTML),
    'Copied as Markdown',
  );
}

export async function copyElementAsImageToClipboard(
  element: HTMLElement,
): Promise<void> {
  if (!navigator.clipboard?.write) {
    showToast('error', 'Image copy needs a secure (https) page');
    return;
  }

  try {
    const blob = await domToBlob(element, { type: 'image/png' });
    if (!blob) {
      showToast('error', 'Could not render that element');
      return;
    }

    await navigator.clipboard.write([new ClipboardItem({ 'image/png': blob })]);
    emitCopy({ format: 'image' });
    showToast('success', 'Copied as image');
  } catch (err) {
    console.error('Failed to copy image:', err);
    showToast('error', 'Could not copy image');
  }
}

export function saveElementAsTextToFile(element: HTMLElement): void {
  const textContent = element.innerText || element.textContent || '';
  if (!textContent.trim()) {
    showToast('info', 'Nothing to save here');
    return;
  }

  const blob = new Blob([textContent.trim()], { type: 'text/plain' });
  const url = URL.createObjectURL(blob);

  const link = document.createElement('a');
  link.href = url;
  link.download = 'clip.txt';
  link.click();

  URL.revokeObjectURL(url);

  showToast('success', 'Saved clip.txt');
}
