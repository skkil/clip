import app from "./state";

import "./styles.css";

document.addEventListener("keydown", (e) => {
  app.handleKeyDown(e);
});

document.addEventListener("keyup", (e) => {
  app.handleKeyUp(e);
});

document.addEventListener("mouseover", (e) => {
  app.handleMouseOver(e);
});

document.addEventListener("mouseout", (e) => {
  app.handleMouseOut(e);
});

document.addEventListener("click", (e) => {
  app.handleClick(e);
});
