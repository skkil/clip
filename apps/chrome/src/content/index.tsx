const keys = {
  alt: false,
};

const elements = {
  hovered: null as HTMLElement | null,
};

document.addEventListener("keydown", (e) => {
  if (e.key === "Alt") {
    keys.alt = true;
    document.body.style.cursor = "copy";
  }
});

document.addEventListener("keyup", (e) => {
  if (e.key === "Alt") {
    keys.alt = false;
    document.body.style.cursor = "default";

    if (elements.hovered) {
      elements.hovered.style.outline = "";
      elements.hovered = null;
    }
  }
});

document.addEventListener("mouseover", (e) => {
  if (keys.alt) {
    if (elements.hovered && elements.hovered !== e.target) {
      elements.hovered.style.outline = "";
    }

    elements.hovered = e.target as HTMLElement;
    elements.hovered.style.outline = "1px solid red";
  }
});

document.addEventListener("mouseout", (e) => {
  if (keys.alt && elements.hovered && e.target === elements.hovered) {
    elements.hovered.style.outline = "";
  }
});

document.addEventListener("click", (e) => {
  if (keys.alt) {
    e.preventDefault();
    e.stopPropagation();

    const element = e.target as HTMLElement;
    const textContent = element.innerText || element.textContent || "";

    if (textContent.trim()) {
      navigator.clipboard.writeText(textContent.trim()).catch((err) => {
        console.error("Failed to copy text:", err);
      });
    }
  }
});
