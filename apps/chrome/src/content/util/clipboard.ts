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
