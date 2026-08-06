import { MessageType } from '@/shared/messages';
import {
  getPreferences,
  registerPreferencesChangeEventListener,
} from '@/shared/preferences';
import { getSite, isEnabledOn } from '@/shared/sites';

const BADGE_TEXT = 'off';

void chrome.action.setBadgeBackgroundColor({ color: '#6b6b6b' });
void chrome.action.setBadgeTextColor({ color: '#ffffff' });

function setBadge(tabId: number, active: boolean): void {
  void chrome.action
    .setBadgeText({ tabId, text: active ? '' : BADGE_TEXT })
    .catch(() => undefined);
}

async function refreshBadge(
  tabId: number,
  url: string | undefined,
): Promise<void> {
  const preferences = await getPreferences();
  setBadge(tabId, isEnabledOn(preferences, getSite(url)));
}

async function refreshAllBadges(): Promise<void> {
  const [preferences, tabs] = await Promise.all([
    getPreferences(),
    chrome.tabs.query({}),
  ]);

  for (const tab of tabs) {
    if (tab.id !== undefined) {
      setBadge(tab.id, isEnabledOn(preferences, getSite(tab.url)));
    }
  }
}

chrome.runtime.onInstalled.addListener(({ reason }) => {
  void refreshAllBadges();

  if (reason !== chrome.runtime.OnInstalledReason.INSTALL) {
    return;
  }

  chrome.tabs.create({ url: chrome.runtime.getURL('welcome.html') });
});

chrome.runtime.onStartup.addListener(() => {
  void refreshAllBadges();
});

chrome.tabs.onUpdated.addListener((tabId, changeInfo, tab) => {
  if (!changeInfo.url && changeInfo.status !== 'complete') {
    return;
  }

  void refreshBadge(tabId, changeInfo.url ?? tab.url);
});

chrome.tabs.onActivated.addListener(({ tabId }) => {
  chrome.tabs.get(tabId, (tab) => {
    if (chrome.runtime.lastError || !tab) {
      return;
    }

    void refreshBadge(tabId, tab.url);
  });
});

registerPreferencesChangeEventListener(() => {
  void refreshAllBadges();
});

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
