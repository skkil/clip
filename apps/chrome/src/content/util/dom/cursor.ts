interface OriginalCursor {
  value: string;
  priority: string;
}

let original: OriginalCursor | null = null;

export function setCursor(cursor: string): void {
  if (original === null) {
    original = {
      value: document.body.style.getPropertyValue('cursor'),
      priority: document.body.style.getPropertyPriority('cursor'),
    };
  }

  document.body.style.setProperty('cursor', cursor);
}

export function restoreCursor(): void {
  if (original === null) {
    return;
  }

  if (original.value) {
    document.body.style.setProperty(
      'cursor',
      original.value,
      original.priority,
    );
  } else {
    document.body.style.removeProperty('cursor');
  }

  original = null;
}
