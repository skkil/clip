import {
  Toast,
  type ToastMessage,
  type ToastStatus,
} from '@/content/components/toast/Toast';
import css from '@/content/components/toast/Toast.module.css?inline';
import {
  DEFAULT_PREFERENCES,
  getPreferences,
  registerPreferencesChangeEventListener,
} from '@/shared/preferences';

import { type Mount, mount } from './dom/mount';

const DURATION = 2400;

let enabled = DEFAULT_PREFERENCES.toasts.enabled;

let mounted: Mount | null = null;
let current: ToastMessage | null = null;
let visible = false;
let timer: number | null = null;

getPreferences().then((preferences) => {
  enabled = preferences.toasts.enabled;
});

registerPreferencesChangeEventListener((preferences) => {
  enabled = preferences.toasts.enabled;
});

function paint(next: boolean): void {
  visible = next;
  mounted?.render(<Toast message={current} visible={next} />);
}

function show(status: ToastStatus, text: string): void {
  current = { status, text };

  if (!mounted) {
    mounted = mount(<Toast message={current} visible={false} />, css);
    visible = false;
  }

  if (visible) {
    paint(true);
  } else {
    paint(false);
    requestAnimationFrame(() => {
      requestAnimationFrame(() => {
        paint(true);
      });
    });
  }

  if (timer !== null) {
    window.clearTimeout(timer);
  }

  timer = window.setTimeout(() => {
    timer = null;
    paint(false);
  }, DURATION);
}

async function isEnabled(): Promise<boolean> {
  try {
    enabled = (await getPreferences()).toasts.enabled;
  } catch (err) {
    console.error('Failed to read toast preference:', err);
  }

  return enabled;
}

export function showToast(status: ToastStatus, text: string): void {
  void isEnabled().then((on) => {
    if (on) {
      show(status, text);
    }
  });
}

export function previewToast(status: ToastStatus, text: string): void {
  show(status, text);
}
