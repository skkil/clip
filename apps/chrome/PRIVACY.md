# CLIP Privacy Policy

_Last updated: 2026-08-03_

CLIP is a browser extension that lets you copy elements from any web page as
text, HTML, Markdown, or an image, and take screenshots of page regions.

## What CLIP does not do

- CLIP does not collect, transmit, or store any data on remote servers.
- CLIP does not track your browsing activity.
- CLIP does not include analytics, telemetry, or third-party tracking of any
  kind.

## What CLIP stores locally

- Your preferences (such as your chosen leader key) are stored using the
  `chrome.storage.sync` API, which is synced across your own signed-in Chrome
  browsers by Google — CLIP itself never sees or transmits this data.
- Content you copy is written directly to your system clipboard via the
  browser's Clipboard API and is never sent anywhere else.

## Why CLIP requests broad permissions

- `<all_urls>` / host permissions: required so the copy and screenshot
  features work on any page you visit. CLIP only reads the DOM of the page
  you are actively interacting with, on demand, and never in the background.
- `clipboardWrite`: required to copy content to your system clipboard.
- `storage`: required to save your local preferences.
- `tabs`: required to capture a screenshot of the visible tab when you use
  the screenshot feature.

## Contact

Questions about this policy can be directed to [your contact email/URL here].
