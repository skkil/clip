import type { ReactNode } from 'react';
import { createRoot } from 'react-dom/client';

export interface Mount {
  render: (element: ReactNode) => void;
  unmount: () => void;
}

export function mount(element: ReactNode, css: string): Mount {
  const host = document.createElement('div');
  document.body.appendChild(host);

  const shadow = host.attachShadow({ mode: 'open' });

  const style = document.createElement('style');
  style.textContent = css;
  shadow.appendChild(style);

  const mountPoint = document.createElement('div');
  shadow.appendChild(mountPoint);

  const root = createRoot(mountPoint);
  root.render(element);

  return {
    render: (next: ReactNode) => {
      root.render(next);
    },
    unmount: () => {
      root.unmount();
      host.remove();
    },
  };
}
