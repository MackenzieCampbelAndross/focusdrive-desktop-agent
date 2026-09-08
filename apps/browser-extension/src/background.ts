// FocusDrive Browser Companion Extension (Chrome & Edge Manifest V3)
const NATIVE_HOST_NAME = 'com.focusdrive.desktopagent.native';

let port: chrome.runtime.Port | null = null;
let lastReportedDomain: string | null = null;

function getNativePort(): chrome.runtime.Port {
  if (!port) {
    try {
      port = chrome.runtime.connectNative(NATIVE_HOST_NAME);
      port.onDisconnect.addListener(() => {
        console.warn('Disconnected from FocusDrive Native Agent host:', chrome.runtime.lastError?.message);
        port = null;
      });
      port.onMessage.addListener((msg) => {
        console.log('Received response from FocusDrive Native Host:', msg);
      });
    } catch (e) {
      console.error('Failed to connect to native messaging host:', e);
      port = null;
    }
  }
  return port;
}

function sendActiveTabDomain(rawUrl: string | undefined, browserName: 'chrome' | 'edge') {
  if (!rawUrl) return;

  try {
    const urlObj = new URL(rawUrl);
    // Ignore internal browser protocol pages (chrome://, edge://, about:)
    if (urlObj.protocol !== 'http:' && urlObj.protocol !== 'https:') {
      return;
    }

    const domain = urlObj.hostname.toLowerCase();
    if (domain === lastReportedDomain) {
      return;
    }

    lastReportedDomain = domain;

    const payload = {
      type: 'tab_active',
      browser: browserName,
      domain: domain,
      timestamp: new Date().toISOString(),
    };

    const nativePort = getNativePort();
    if (nativePort) {
      nativePort.postMessage(payload);
      console.log('Observed active tab domain:', domain);
    }
  } catch (err) {
    // Ignore invalid URLs
  }
}

function checkCurrentActiveTab() {
  const browserName = typeof (globalThis as any).chrome !== 'undefined' ? 'chrome' : 'edge';
  chrome.tabs.query({ active: true, currentWindow: true }, (tabs) => {
    if (tabs && tabs.length > 0 && tabs[0].url) {
      sendActiveTabDomain(tabs[0].url, browserName);
    }
  });
}

// Observe active tab switches
chrome.tabs.onActivated.addListener((_activeInfo) => {
  checkCurrentActiveTab();
});

// Observe URL navigation in active tab
chrome.tabs.onUpdated.addListener((_tabId, changeInfo, tab) => {
  if (changeInfo.status === 'complete' && tab.active && tab.url) {
    const browserName = typeof (globalThis as any).chrome !== 'undefined' ? 'chrome' : 'edge';
    sendActiveTabDomain(tab.url, browserName);
  }
});

// Periodically check active tab
setInterval(checkCurrentActiveTab, 5000);

console.log('FocusDrive Browser Companion Extension Initialized');
