/**
 * API Integration with Tauri Bridge
 *
 * This module demonstrates how to integrate the Open CoreUI frontend
 * with the new Tauri integrated backend while maintaining compatibility
 * with both desktop and web deployment targets.
 */

import { invoke } from '@tauri-apps/api/core';
import { isTauri } from '@tauri-apps/api/core';
import { bridgeAPI } from './tauri-bridge.js';

/**
 * API Client that works with both Tauri integrated backend and web deployment
 */
class OpenCoreUIAPIClient {
    constructor() {
        this.isTauri = isTauri();
        this.baseUrl = this.isTauri ? '' : 'http://localhost:8168';
        this.initialized = false;

        console.log(`🔌 API Client initialized for ${this.isTauri ? 'Tauri integrated' : 'web'} mode`);
    }

    /**
     * Initialize the API client
     */
    async initialize() {
        if (this.initialized) {
            return true;
        }

        try {
            if (this.isTauri) {
                // Wait for Tauri bridge to be ready
                await this.waitForBridgeReady();

                // Test backend connection
                await this.testConnection();
            }

            this.initialized = true;
            console.log('✅ API Client initialization completed');
            return true;
        } catch (error) {
            console.error('❌ API Client initialization failed:', error);
            return false;
        }
    }

    /**
     * Wait for the Tauri bridge to be ready
     */
    async waitForBridgeReady(timeout = 10000) {
        return new Promise((resolve, reject) => {
            if (!this.isTauri) {
                resolve();
                return;
            }

            const timeoutId = setTimeout(() => {
                reject(new Error('Bridge ready timeout'));
            }, timeout);

            const checkReady = () => {
                if (window.TauriBridge && window.TauriBridge.CONFIG.enabled) {
                    clearTimeout(timeoutId);
                    resolve();
                } else {
                    setTimeout(checkReady, 100);
                }
            };

            checkReady();
        });
    }

    /**
     * Test the backend connection
     */
    async testConnection() {
        try {
            const health = await this.get('/health');
            console.log('✅ Backend connection test passed:', health);
            return true;
        } catch (error) {
            console.error('❌ Backend connection test failed:', error);
            throw error;
        }
    }

    /**
     * Generic HTTP request method
     */
    async request(path, options = {}) {
        await this.initialize();

        const url = `${this.baseUrl}${path}`;

        if (this.isTauri) {
            return this.tauriRequest(path, options);
        } else {
            return this.webRequest(url, options);
        }
    }

    /**
     * Tauri-specific request handling
     */
    async tauriRequest(path, options) {
        const method = options.method || 'GET';
        const headers = options.headers || {};
        const body = options.body;

        // Create LocalRequest format
        const localRequest = {
            uri: path,
            method,
            body: body ? (typeof body === 'string' ? body : JSON.stringify(body)) : null,
            headers
        };

        // Use Tauri bridge
        try {
            const localResponse = await invoke('handle_http_request', {
                request: localRequest
            });

            // Convert to standard Response-like object
            return {
                ok: localResponse.status_code >= 200 && localResponse.status_code < 300,
                status: localResponse.status_code,
                statusText: this.getStatusText(localResponse.status_code),
                headers: localResponse.headers,
                json: async () => JSON.parse(new TextDecoder().decode(localResponse.body)),
                text: async () => new TextDecoder().decode(localResponse.body)
            };
        } catch (error) {
            throw new Error(`Tauri bridge request failed: ${error.message}`);
        }
    }

    /**
     * Web-specific request handling (fallback)
     */
    async webRequest(url, options) {
        try {
            const response = await fetch(url, options);

            // Return response-like object
            return {
                ok: response.ok,
                status: response.status,
                statusText: response.statusText,
                headers: Object.fromEntries(response.headers.entries()),
                json: async () => response.json(),
                text: async () => response.text()
            };
        } catch (error) {
            throw new Error(`Web request failed: ${error.message}`);
        }
    }

    /**
     * HTTP method helpers
     */
    async get(path, headers = {}) {
        return this.request(path, { method: 'GET', headers });
    }

    async post(path, data, headers = {}) {
        return this.request(path, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                ...headers
            },
            body: JSON.stringify(data)
        });
    }

    async put(path, data, headers = {}) {
        return this.request(path, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
                ...headers
            },
            body: JSON.stringify(data)
        });
    }

    async delete(path, headers = {}) {
        return this.request(path, { method: 'DELETE', headers });
    }

    /**
     * Open CoreUI specific API methods
     */
    async getConfig() {
        try {
            if (this.isTauri) {
                return await bridgeAPI.getConfig();
            } else {
                const response = await this.get('/api/config');
                return await response.json();
            }
        } catch (error) {
            console.error('Failed to get config:', error);
            throw error;
        }
    }

    async getModels() {
        try {
            if (this.isTauri) {
                return await bridgeAPI.getModels();
            } else {
                const response = await this.get('/api/models');
                return await response.json();
            }
        } catch (error) {
            console.error('Failed to get models:', error);
            throw error;
        }
    }

    async chatCompletion(payload, userId = null) {
        try {
            if (this.isTauri) {
                return await bridgeAPI.chatCompletion(payload, userId);
            } else {
                const response = await this.post('/api/chat/completions', payload);
                return await response.json();
            }
        } catch (error) {
            console.error('Chat completion failed:', error);
            throw error;
        }
    }

    async authenticate(credentials) {
        try {
            const response = await this.post('/api/auth/login', credentials);
            const result = await response.json();

            if (response.ok && result.token) {
                this.setAuthToken(result.token);
            }

            return result;
        } catch (error) {
            console.error('Authentication failed:', error);
            throw error;
        }
    }

    async register(userData) {
        try {
            const response = await this.post('/api/auth/register', userData);
            return await response.json();
        } catch (error) {
            console.error('Registration failed:', error);
            throw error;
        }
    }

    async getUserProfile() {
        try {
            const response = await this.get('/api/user/profile', this.getAuthHeaders());
            return await response.json();
        } catch (error) {
            console.error('Failed to get user profile:', error);
            throw error;
        }
    }

    async uploadFile(file, metadata = {}) {
        try {
            const formData = new FormData();
            formData.append('file', file);

            // Add metadata
            Object.entries(metadata).forEach(([key, value]) => {
                formData.append(key, value);
            });

            const response = await this.request('/api/upload', {
                method: 'POST',
                body: formData,
                headers: this.getAuthHeaders()
                // Note: Don't set Content-Type for FormData, browser sets it with boundary
            });

            return await response.json();
        } catch (error) {
            console.error('File upload failed:', error);
            throw error;
        }
    }

    /**
     * WebSocket and real-time features
     */
    createSocketConnection(path = '/socket.io/') {
        if (this.isTauri) {
            // In Tauri, the bridge automatically routes WebSocket connections
            return io(path);
        } else {
            // In web mode, connect to the backend server
            return io(this.baseUrl + path);
        }
    }

    /**
     * Authentication helpers
     */
    setAuthToken(token) {
        localStorage.setItem('authToken', token);
    }

    getAuthToken() {
        return localStorage.getItem('authToken');
    }

    clearAuthToken() {
        localStorage.removeItem('authToken');
    }

    getAuthHeaders() {
        const token = this.getAuthToken();
        return token ? { 'Authorization': `Bearer ${token}` } : {};
    }

    /**
     * Utility methods
     */
    getStatusText(statusCode) {
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
     * Health check
     */
    async healthCheck() {
        try {
            if (this.isTauri) {
                return await bridgeAPI.healthCheck();
            } else {
                const response = await this.get('/health');
                return await response.json();
            }
        } catch (error) {
            console.error('Health check failed:', error);
            return { status: false, error: error.message };
        }
    }
}

// Create singleton instance
const apiClient = new OpenCoreUIAPIClient();

// Export for use in application
export default apiClient;

// Export convenience functions
export const {
    getConfig,
    getModels,
    chatCompletion,
    authenticate,
    register,
    getUserProfile,
    uploadFile,
    createSocketConnection,
    healthCheck
} = apiClient;

// Export class for creating additional instances
export { OpenCoreUIAPIClient };

// Auto-initialize
apiClient.initialize().catch(console.error);