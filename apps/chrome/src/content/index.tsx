import '@/shared/styles/global.css';

import app from './state';

document.addEventListener('keydown', (e) => {
  app.handleKeyDown(e);
});

document.addEventListener('keyup', (e) => {
  app.handleKeyUp(e);
});

document.addEventListener('mouseover', (e) => {
  app.handleMouseOver(e);
});

document.addEventListener('mouseout', (e) => {
  app.handleMouseOut(e);
});

document.addEventListener('mousedown', (e) => {
  app.handleMouseDown(e);
});

document.addEventListener('mousemove', (e) => {
  app.handleMouseMove(e);
});

document.addEventListener('mouseup', (e) => {
  app.handleMouseUp(e);
});

document.addEventListener('click', (e) => {
  app.handleClick(e);
});
