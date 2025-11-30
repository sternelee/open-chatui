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

import { invoke } from '@tauri-apps/api/core';
import { isTauri } from '@tauri-apps/api/core';

// Global configuration
const CONFIG = {
    // Whether to use the bridge (auto-detected)
    enabled: isTauri(),

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

/**
 * Request queue for handling concurrent requests
 */
class RequestQueue {
    constructor() {
        this.pending = new Map();
        this.requestId = 0;
    }

    generateId() {
        return `req_${++this.requestId}_${Date.now()}`;
    }

    add(request) {
        const id = this.generateId();
        this.pending.set(id, { request, resolve: null, reject: null });
        return id;
    }

    resolve(id, response) {
        const item = this.pending.get(id);
        if (item && item.resolve) {
            item.resolve(response);
        }
        this.pending.delete(id);
    }

    reject(id, error) {
        const item = this.pending.get(id);
        if (item && item.reject) {
            item.reject(error);
        }
        this.pending.delete(id);
    }
}

const requestQueue = new RequestQueue();

/**
 * Convert fetch Request to LocalRequest format
 */
function requestToLocalRequest(request, url) {
    const headers = {};
    request.headers.forEach((value, key) => {
        headers[key.toLowerCase()] = value;
    });

    return {
        uri: url + request.url.substring(window.location.origin.length),
        method: request.method,
        body: request.method !== 'GET' && request.method !== 'HEAD' ?
               request.text() : null,
        headers: headers
    };
}

/**
 * Convert LocalResponse to fetch Response
 */
function localResponseToResponse(localResponse) {
    const headers = new Headers();
    Object.entries(localResponse.headers).forEach(([key, value]) => {
        headers.set(key, value);
    });

    return new Response(localResponse.body, {
        status: localResponse.status_code,
        statusText: getStatusText(localResponse.status_code),
        headers: headers
    });
}

/**
 * Get status text for HTTP status codes
 */
function getStatusText(statusCode) {
    const statusTexts = {
        200: 'OK',
        201: 'Created',
        204: 'No Content',
        400: 'Bad Request',
        401: 'Unauthorized',
        403: 'Forbidden',
        404: 'Not Found',
        500: 'Internal Server Error',
        502: 'Bad Gateway',
        503: 'Service Unavailable'
    };

    return statusTexts[statusCode] || 'Unknown';
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

/**
 * Intercept and handle HTTP request through Tauri bridge
 */
async function handleRequestWithBridge(input, init = {}) {
    try {
        const request = new Request(input, init);
        const url = request.url;

        // Create LocalRequest
        const body = init.body ?
            (typeof init.body === 'string' ? init.body : JSON.stringify(init.body)) :
            null;

        const localRequest = {
            uri: url.substring(window.location.origin.length),
            method: request.method,
            body: body,
            headers: Object.fromEntries(request.headers.entries())
        };

        // Send request through Tauri bridge
        const localResponse = await invoke('handle_http_request', {
            request: localRequest
        });

        // Convert back to Response
        return localResponseToResponse(localResponse);

    } catch (error) {
        console.error('Bridge request failed:', error);

        // Fallback to regular fetch if bridge fails
        if (CONFIG.fallbackBaseUrl) {
            try {
                const fallbackUrl = new URL(input, CONFIG.fallbackBaseUrl);
                return fetch(fallbackUrl, init);
            } catch (fallbackError) {
                console.error('Fallback request also failed:', fallbackError);
            }
        }

        throw error;
    }
}

/**
 * Enhanced fetch function with automatic bridge detection
 */
async function tauriFetch(input, init = {}) {
    // If not in Tauri environment or bridge is disabled, use regular fetch
    if (!CONFIG.enabled) {
        return fetch(input, init);
    }

    // Convert input to URL string for consistent handling
    const urlString = typeof input === 'string' ? input : input.url;

    // Check if we should use the bridge for this request
    if (shouldUseBridge(urlString)) {
        return handleRequestWithBridge(input, init);
    }

    // Use regular fetch for non-bridge requests
    return fetch(input, init);
}

/**
 * Patch global fetch to use our bridge
 */
function patchGlobalFetch() {
    if (!CONFIG.enabled) {
        console.log('🌐 Tauri bridge disabled, using regular fetch');
        return;
    }

    const originalFetch = window.fetch;

    window.fetch = function(input, init = {}) {
        return tauriFetch(input, init);
    };

    // Preserve original fetch for direct access
    window.originalFetch = originalFetch;

    console.log('🌉 Tauri bridge initialized and fetch patched');
}

/**
 * Initialize the bridge when DOM is ready
 */
function initializeBridge() {
    // Wait for DOM to be ready
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', initializeBridge);
        return;
    }

    console.log('🚀 Initializing Tauri bridge for Open CoreUI...');

    // Patch global fetch
    patchGlobalFetch();

    // Test the bridge with a health check
    setTimeout(async () => {
        try {
            const response = await tauriFetch('/health');
            const data = await response.json();
            console.log('✅ Bridge test successful:', data);

            // Emit custom event to notify application
            window.dispatchEvent(new CustomEvent('tauri-bridge-ready', {
                detail: { success: true, data }
            }));
        } catch (error) {
            console.error('❌ Bridge test failed:', error);

            // Emit error event
            window.dispatchEvent(new CustomEvent('tauri-bridge-error', {
                detail: { success: false, error: error.message }
            }));
        }
    }, 1000);
}

/**
 * Direct access functions for specific API endpoints
 */
export const bridgeAPI = {
    /**
     * Get application configuration
     */
    async getConfig() {
        if (!CONFIG.enabled) {
            return fetch('/api/config').then(r => r.json());
        }

        try {
            const response = await invoke('get_backend_config');
            return response;
        } catch (error) {
            console.error('Failed to get config via bridge:', error);
            throw error;
        }
    },

    /**
     * Get available models
     */
    async getModels() {
        if (!CONFIG.enabled) {
            return fetch('/api/models').then(r => r.json());
        }

        try {
            const response = await invoke('get_backend_models');
            return response;
        } catch (error) {
            console.error('Failed to get models via bridge:', error);
            throw error;
        }
    },

    /**
     * Send chat completion request
     */
    async chatCompletion(payload, userId = null) {
        if (!CONFIG.enabled) {
            return fetch('/api/chat/completions', {
                method: 'POST',
                headers: { 'Content-Type': 'application/json' },
                body: JSON.stringify(payload)
            }).then(r => r.json());
        }

        try {
            const response = await invoke('chat_completion', {
                payload,
                userId
            });
            return response;
        } catch (error) {
            console.error('Chat completion failed via bridge:', error);
            throw error;
        }
    },

    /**
     * Health check
     */
    async healthCheck() {
        if (!CONFIG.enabled) {
            return fetch('/health').then(r => r.json());
        }

        try {
            const response = await invoke('health_check');
            return response;
        } catch (error) {
            console.error('Health check failed via bridge:', error);
            throw error;
        }
    },

    /**
     * Initialize the backend
     */
    async initializeBackend() {
        if (!CONFIG.enabled) {
            return { success: true, message: 'Not in Tauri environment' };
        }

        try {
            const response = await invoke('initialize_bridge_and_backend');
            return { success: true, message: response };
        } catch (error) {
            console.error('Backend initialization failed:', error);
            return { success: false, error: error.message };
        }
    }
};

// Auto-initialize
if (typeof window !== 'undefined') {
    initializeBridge();
}

// Export for use in other modules
export default {
    tauriFetch,
    bridgeAPI,
    patchGlobalFetch,
    CONFIG
};

// Also provide global access
window.TauriBridge = {
    tauriFetch,
    bridgeAPI,
    CONFIG
};