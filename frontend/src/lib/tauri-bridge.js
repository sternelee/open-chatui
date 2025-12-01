/**
 * Tauri Bridge for Open CoreUI
 *
 * This module provides a bridge between the frontend and the integrated Rust backend,
 * eliminating the need for a separate sidecar process.
 *
 * Features:
 * - Automatic HTTP request interception and forwarding
 * - WebSocket and Socket.IO support through Tauri
 * - Fallback to regular HTTP requests when not in Tauri environment
 */

const { invoke } = window.__TAURI__.core;

let localAppRequestCommand = "handle_http_request";

// Global configuration
const CONFIG = {
    // Whether to use the bridge (auto-detected)
    enabled: window.__TAURI__ ? true : false,

    // Base URL for fallback requests
    fallbackBaseUrl: 'http://localhost:8168',

    // Paths that should always use the bridge
    bridgePaths: [
        '/api/config',
        '/api/models',
        '/api/chat/completions',
        '/api/embeddings',
        '/api/auth/login',
        '/api/auth/register',
        '/api/user/profile',
        '/api/health',
        '/health',
        '/openai/',
        '/socket.io/',
        '/ws/socket.io/'
    ],

    // Paths that should never use the bridge
    excludePaths: [
        '/static/',
        '/assets/',
        '/favicon',
        '/manifest.json'
    ]
};

function initialize(initialPath, localAppRequestCommandOverride) {
  if (localAppRequestCommandOverride) {
    localAppRequestCommand = localAppRequestCommandOverride;
  }

  proxyFetch();
}

function proxyFetch() {
  const originalFetch = window.fetch;

  window.fetch = async function (...args) {
    const [url, options] = args;

    // Check if we should use the bridge for this request
    if (!CONFIG.enabled || !shouldUseBridge(url)) {
      console.log('🔗 Using original fetch for:', url);
      return originalFetch(...args);
    }

    console.log('🌉 Using Tauri bridge for:', url);

    if (url.startsWith("ipc://")) {
      return originalFetch(...args);
    }

    const request = {
      uri: url.startsWith('http') ? new URL(url).pathname + new URL(url).search : url,
      method: options?.method || "GET",
      headers: options?.headers || {},
      ...(options?.body && { body: options.body }),
    };

    let response = await invoke(localAppRequestCommand, {
      localRequest: request,
    });

    // Handle redirects
    while ([301, 302, 303, 307, 308].includes(parseInt(response.status_code))) {
      const location = response.headers["location"];

      const redirectRequest = {
        uri: location,
        method: "GET",
        headers: {},
      };
      response = await invoke(localAppRequestCommand, {
        localRequest: redirectRequest,
      });
    }

    // Convert response.body (which is a number array) to Uint8Array, then to text
    console.log('🔍 Raw Tauri response:', response);

    let bodyText;
    if (Array.isArray(response.body)) {
      console.log('🔧 Converting byte array to text, length:', response.body.length);
      const bodyByteArray = new Uint8Array(response.body);
      const decoder = new TextDecoder("utf-8");
      bodyText = decoder.decode(bodyByteArray);
      console.log('✅ Converted body text:', bodyText);
    } else if (typeof response.body === 'string') {
      console.log('📝 Body is already string:', response.body);
      bodyText = response.body;
    } else if (response.body && typeof response.body === 'object') {
      // Handle nested structure
      console.log('🏗️ Body is object:', response.body);
      bodyText = JSON.stringify(response.body);
    } else {
      console.log('⚠️ Unknown body format, using empty string:', typeof response.body, response.body);
      bodyText = response.body || '';
    }

    const status = parseInt(response.status_code);
    const headers = new Headers(response.headers);

    // Log final response
    console.log('🌐 Final response:', {
      status,
      headers: Object.fromEntries(headers.entries()),
      bodyPreview: bodyText.substring(0, 200) + (bodyText.length > 200 ? '...' : '')
    });

    // Create response and test if it's correctly parsed
    const finalResponse = new Response(bodyText, { status, headers });

    // Test if JSON parsing works
    if (headers.get('content-type')?.includes('application/json')) {
      try {
        const parsed = await finalResponse.clone().json();
        console.log('✅ JSON parsing successful:', parsed);
      } catch (e) {
        console.error('❌ JSON parsing failed:', e);
        console.error('Body that failed to parse:', bodyText);
      }
    }

    return finalResponse;
  };

  // Preserve original fetch for direct access
  window.originalFetch = originalFetch;
  console.log('🌉 Tauri bridge initialized and fetch patched');
}

/**
 * Check if a URL should use the bridge
 */
function shouldUseBridge(url) {
    try {
        const urlObj = new URL(url, window.location.origin);
        const pathname = urlObj.pathname;

        // Check excluded paths first
        for (const excludePath of CONFIG.excludePaths) {
            if (pathname.startsWith(excludePath)) {
                return false;
            }
        }

        // Check bridge paths
        for (const bridgePath of CONFIG.bridgePaths) {
            if (pathname.startsWith(bridgePath) || pathname === bridgePath) {
                return true;
            }
        }

        // Default to bridge for API paths
        return pathname.startsWith('/api/') || pathname.startsWith('/ws/');
    } catch (e) {
        console.warn('Invalid URL for bridge check:', url, e);
        return false;
    }
}

// Auto-initialize when DOM is ready
if (typeof window !== 'undefined') {
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => {
            console.log('🚀 Initializing Tauri bridge for Open CoreUI...');
            initialize();
        });
    } else {
        console.log('🚀 Initializing Tauri bridge for Open CoreUI...');
        initialize();
    }
}

// Export for use in other modules
export default {
    initialize,
    CONFIG
};

// Also provide global access
window.TauriBridge = {
    initialize,
    CONFIG
};

// BEGIN XHR-FETCH-PROXY
(function (originalXMLHttpRequest) {
    class EventTarget {
        constructor() {
            this.eventListeners = {};
        }

        addEventListener(event, callback) {
            if (!this.eventListeners[event]) {
                this.eventListeners[event] = [];
            }
            this.eventListeners[event].push(callback);
        }

        removeEventListener(event, callback) {
            if (!this.eventListeners[event]) return;
            const index = this.eventListeners[event].indexOf(callback);
            if (index !== -1) {
                this.eventListeners[event].splice(index, 1);
            }
        }

        _triggerEvent(event, ...args) {
            if (this.eventListeners[event]) {
                this.eventListeners[event].forEach(callback => callback.apply(this, args));
            }
        }
    }


    class ProxyXMLHttpRequest extends EventTarget {
        constructor() {
            super();
            this.onload = null;
            this.onerror = null;
            this.onreadystatechange = null;

            this.readyState = 0;
            this.status = 0;
            this.statusText = '';
            this.response = null;
            this.responseText = null;
            this.responseType = '';
            this.responseURL = '';
            this.method = null;
            this.url = null;
            this.async = true;
            this.requestHeaders = {};
            this.controller = new AbortController(); // to handle aborts
            this.eventListeners = {};
            this.upload = new EventTarget(); // Adding upload event listeners
            this._triggerEvent('readystatechange');
        }

        open(method, url, async = true, user = null, password = null) {
            this.method = method;
            this.url = url;
            this.async = async;
            this.user = user;
            this.password = password;
            this.readyState = 1;
            this._triggerEvent('readystatechange');
        }

        send(data = null) {
            const options = {
                method: this.method,
                headers: this.requestHeaders,
                body: data,
                signal: this.controller.signal,
                mode: 'cors',
                credentials: this.user || this.password ? 'include' : 'same-origin',
            };

            if (this.user && this.password) {
                const base64Credentials = btoa(`${this.user}:${this.password}`);
                options.headers['Authorization'] = `Basic ${base64Credentials}`;
            }

            this.readyState = 2;
            this._triggerEvent('readystatechange');

            fetch(this.url, options)
                .then(response => {
                    this.status = response.status;
                    this.statusText = response.statusText;
                    this.responseURL = response.url;
                    this._parseHeaders(response.headers);

                    this.readyState = 3;
                    this._triggerEvent('readystatechange');
                    return this._parseResponse(response);
                })
                .then(responseData => {
                    this.readyState = 4;
                    this.response = responseData;
                    this.responseText = typeof responseData === 'string' ? responseData : JSON.stringify(responseData);
                    this._triggerEvent('readystatechange');
                    if (this.onload) this.onload();
                })
                .catch(error => {
                    if (this.onerror) this.onerror(error);
                });
        }

        setRequestHeader(header, value) {
            this.requestHeaders[header] = value;
        }

        abort() {
            this.controller.abort();
            this.readyState = 0;
            this._triggerEvent('readystatechange');
        }

        getResponseHeader(header) {
            return this.responseHeaders[header.toLowerCase()] || null;
        }

        getAllResponseHeaders() {
            return Object.entries(this.responseHeaders)
                .map(([key, value]) => `${key}: ${value}`)
                .join('\r\n');
        }

        overrideMimeType(mime) {
            this.overrideMime = mime;
        }

        _parseHeaders(headers) {
            this.responseHeaders = {};
            headers.forEach((value, key) => {
                this.responseHeaders[key.toLowerCase()] = value;
            });
        }

        _parseResponse(response) {
            const contentType = response.headers.get('content-type');
            if (contentType && contentType.includes('application/json')) {
                return response.json();
            } else if (contentType && (contentType.includes('text/') || contentType.includes('xml'))) {
                return response.text();
            } else {
                return response.blob(); // default to blob for binary data
            }
        }

        _triggerEvent(event, ...args) {
            super._triggerEvent(event, ...args);
            if (this[`on${event}`]) {
                this[`on${event}`].apply(this, args);
            }

            if (event.startsWith('progress') || event === 'loadstart' || event === 'loadend' || event === 'abort') {
                this.upload._triggerEvent(event, ...args);
            }
        }
    }

    window.XMLHttpRequest = ProxyXMLHttpRequest;
})(window.XMLHttpRequest);
// END XHR-FETCH-PROXY