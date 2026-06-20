import { Key } from "../types/key";

export function getKey(key: string): Key | undefined {
  switch (key) {
    case "Alt":
      return Key.ALT;
    case "Control":
      return Key.CONTROL;
    case "Shift":
      return Key.SHIFT;
    case "Escape":
      return Key.ESCAPE;
    case "Enter":
      return Key.ENTER;
  }
}
