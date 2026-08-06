import { Key } from '@/shared/types/key';

export function getKey(key: string): Key | undefined {
  switch (key) {
    case 'Alt':
      return Key.ALT;
    case 'Control':
      return Key.CONTROL;
    case 'Shift':
      return Key.SHIFT;
    case 'Escape':
      return Key.ESCAPE;
    case 'Enter':
      return Key.ENTER;
  }
}

export function getKeyLabel(key: Key): string {
  switch (key) {
    case Key.ALT:
      return 'Alt';
    case Key.CONTROL:
      return 'Ctrl';
    case Key.SHIFT:
      return 'Shift';
    case Key.ESCAPE:
      return 'Esc';
    case Key.ENTER:
      return 'Enter';
  }
}
