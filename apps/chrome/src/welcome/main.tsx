import { StrictMode } from 'react';
import { createRoot } from 'react-dom/client';

import '@/content';

import Welcome from './Welcome.tsx';
import './index.css';

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <Welcome />
  </StrictMode>,
);
