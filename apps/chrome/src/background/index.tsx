import { MessageType } from '@/shared/messages';

chrome.runtime.onMessage.addListener((message, sender, sendResponse) => {
  if (message?.type !== MessageType.CAPTURE_SELECTION) {
    return;
  }

  if (!sender.tab?.windowId) {
    sendResponse({ error: 'No window to capture' });
    return;
  }

  chrome.tabs.captureVisibleTab(
    sender.tab.windowId,
    { format: 'png' },
    (dataUrl) => {
      if (chrome.runtime.lastError || !dataUrl) {
        sendResponse({ error: chrome.runtime.lastError?.message });
        return;
      }

      sendResponse({ dataUrl });
    },
  );

  return true;
});
