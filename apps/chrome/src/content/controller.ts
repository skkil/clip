import {
  type Preferences,
  getPreferences,
  registerPreferencesChangeEventListener,
} from '@/shared/preferences';
import { getSite, isEnabledOn } from '@/shared/sites';

import app from './state';
import { applyHighlightColor, clearHighlightColor } from './util/highlight';
import { dismissToast } from './util/toast';

const HANDLERS: Record<string, (event: Event) => void> = {
  keydown: (e) => app.handleKeyDown(e as KeyboardEvent),
  keyup: (e) => app.handleKeyUp(e as KeyboardEvent),
  mouseover: (e) => app.handleMouseOver(e as MouseEvent),
  mouseout: (e) => app.handleMouseOut(e as MouseEvent),
  mousedown: (e) => app.handleMouseDown(e as MouseEvent),
  mousemove: (e) => app.handleMouseMove(e as MouseEvent),
  mouseup: (e) => app.handleMouseUp(e as MouseEvent),
  click: (e) => app.handleClick(e as MouseEvent),
};

const LISTENERS = Object.entries(HANDLERS);

let attached = false;

function attach(): void {
  if (attached) {
    return;
  }

  for (const [type, handler] of LISTENERS) {
    document.addEventListener(type, handler);
  }

  attached = true;
}

function detach(): void {
  if (!attached) {
    return;
  }

  for (const [type, handler] of LISTENERS) {
    document.removeEventListener(type, handler);
  }

  attached = false;

  app.reset();
  dismissToast();
  clearHighlightColor();
}

export function start(): void {
  const site = getSite(window.location.href);

  const sync = (preferences: Preferences) => {
    app.applyPreferences(preferences);

    if (!isEnabledOn(preferences, site)) {
      detach();
      return;
    }

    applyHighlightColor(preferences.highlight.color);
    attach();
  };

  getPreferences()
    .then(sync)
    .catch((err) => {
      console.error('CLIP stayed inactive, could not read preferences:', err);
    });

  registerPreferencesChangeEventListener(sync);
}
