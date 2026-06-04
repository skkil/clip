import { domToBlob } from "modern-screenshot";

function copyTextToClipboard(text: string): void {
  if (navigator.clipboard && navigator.clipboard.writeText) {
    navigator.clipboard.writeText(text).catch((err) => {
      console.error("Failed to copy text:", err);
    });
  }
}

export function copyElementAsTextToClipboard(element: HTMLElement): void {
  const textContent = element.innerText || element.textContent || "";
  if (textContent.trim()) {
    copyTextToClipboard(textContent.trim());
  }
}

export function copyElementAsHtmlToClipboard(element: HTMLElement): void {
  copyTextToClipboard(element.outerHTML);
}

export function copyElementAsMarkdownToClipboard(element: HTMLElement): void {
  // Placeholder for Markdown conversion logic
  const markdownContent = `**${element.tagName.toLowerCase()}**: ${element.innerText}`;
  copyTextToClipboard(markdownContent);
}

export async function copyElementAsImageToClipboard(
  element: HTMLElement,
): Promise<void> {
  const blob = await domToBlob(element, { type: "image/png" });
  if (blob) {
    await navigator.clipboard.write([new ClipboardItem({ "image/png": blob })]);
  }
}
