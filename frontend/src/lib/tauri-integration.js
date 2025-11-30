/**
 * Tauri Integration for Open CoreUI
 *
 * This module handles the integration between the frontend and the Tauri desktop app,
 * automatically initializing the backend bridge and handling application lifecycle.
 */

import { invoke } from '@tauri-apps/api/core';
import { isTauri, getVersion } from '@tauri-apps/api/core';
import { bridgeAPI } from './tauri-bridge.js';

class TauriIntegration {
    constructor() {
        this.isInitialized = false;
        this.backendReady = false;
        this.appInfo = {
            version: 'unknown',
            platform: 'web',
            tauri: false
        };
    }

    /**
     * Initialize the Tauri integration
     */
    async initialize() {
        if (this.isInitialized) {
            return;
        }

        console.log('🔧 Initializing Tauri integration...');

        try {
            // Check if we're running in Tauri
            this.appInfo.tauri = isTauri();

            if (this.appInfo.tauri) {
                // Get app version
                this.appInfo.version = await getVersion();

                // Detect platform
                this.appInfo.platform = await this.detectPlatform();

                console.log('✅ Running in Tauri environment');
                console.log(`📱 Version: ${this.appInfo.version}`);
                console.log(`💻 Platform: ${this.appInfo.platform}`);

                // Initialize backend
                await this.initializeBackend();

                // Setup event listeners
                this.setupEventListeners();

                // Override native functions
                this.overrideNativeFunctions();
            } else {
                console.log('🌐 Running in web browser mode');
            }

            this.isInitialized = true;

            // Emit ready event
            window.dispatchEvent(new CustomEvent('tauri-integration-ready', {
                detail: this.appInfo
            }));

        } catch (error) {
            console.error('❌ Failed to initialize Tauri integration:', error);

            // Emit error event
            window.dispatchEvent(new CustomEvent('tauri-integration-error', {
                detail: { error: error.message }
            }));
        }
    }

    /**
     * Initialize the backend bridge
     */
    async initializeBackend() {
        try {
            console.log('🚀 Initializing integrated backend...');

            // Initialize backend through Tauri
            const result = await bridgeAPI.initializeBackend();

            if (result.success) {
                console.log('✅ Backend initialized successfully');
                this.backendReady = true;

                // Test the backend
                await this.testBackend();

                // Emit backend ready event
                window.dispatchEvent(new CustomEvent('tauri-backend-ready'));
            } else {
                throw new Error(result.error);
            }

        } catch (error) {
            console.error('❌ Backend initialization failed:', error);
            throw error;
        }
    }

    /**
     * Test the backend connection
     */
    async testBackend() {
        try {
            console.log('🔍 Testing backend connection...');

            const health = await bridgeAPI.healthCheck();
            console.log('✅ Backend health check passed:', health);

            const config = await bridgeAPI.getConfig();
            console.log('✅ Backend config loaded');

            const models = await bridgeAPI.getModels();
            console.log('✅ Backend models loaded');

            return true;
        } catch (error) {
            console.error('❌ Backend test failed:', error);
            throw error;
        }
    }

    /**
     * Detect the current platform
     */
    async detectPlatform() {
        try {
            const platform = await invoke('get_platform');
            return platform || 'unknown';
        } catch (error) {
            // Fallback to browser detection
            if (navigator.platform.includes('Win')) return 'windows';
            if (navigator.platform.includes('Mac')) return 'macos';
            if (navigator.platform.includes('Linux')) return 'linux';
            return 'web';
        }
    }

    /**
     * Setup event listeners for Tauri events
     */
    setupEventListeners() {
        // Listen for window state changes
        window.addEventListener('tauri://window-created', (event) => {
            console.log('🪟 Tauri window created:', event);
        });

        window.addEventListener('tauri://window-closed', (event) => {
            console.log('🪟 Tauri window closed:', event);
        });

        // Listen for menu events
        window.addEventListener('tauri://menu', (event) => {
            console.log('📋 Tauri menu event:', event);
        });

        // Listen for file drops
        window.addEventListener('tauri://file-drop', (event) => {
            console.log('📁 Files dropped:', event.detail.paths);
            this.handleFileDrop(event.detail.paths);
        });

        // Listen for system theme changes
        window.addEventListener('tauri://theme-changed', (event) => {
            console.log('🎨 Theme changed:', event.detail.theme);
            this.handleThemeChange(event.detail.theme);
        });
    }

    /**
     * Override native functions for better integration
     */
    overrideNativeFunctions() {
        // Override alert for better desktop experience
        const originalAlert = window.alert;
        window.alert = (message) => {
            if (this.appInfo.tauri) {
                console.log('⚠️ Alert:', message);
                // In a real implementation, you might want to use a native dialog
            }
            originalAlert(message);
        };

        // Override confirm for better desktop experience
        const originalConfirm = window.confirm;
        window.confirm = (message) => {
            if (this.appInfo.tauri) {
                console.log('❓ Confirm:', message);
                // In a real implementation, you might want to use a native dialog
            }
            return originalConfirm(message);
        };

        console.log('🔧 Native function overrides applied');
    }

    /**
     * Handle file drop events
     */
    handleFileDrop(paths) {
        // Emit a custom event for the application to handle
        window.dispatchEvent(new CustomEvent('tauri-files-dropped', {
            detail: { paths }
        }));
    }

    /**
     * Handle theme change events
     */
    handleThemeChange(theme) {
        // Update CSS variables or classes
        document.documentElement.setAttribute('data-theme', theme);

        // Emit custom event
        window.dispatchEvent(new CustomEvent('tauri-theme-changed', {
            detail: { theme }
        }));
    }

    /**
     * Get app information
     */
    getAppInfo() {
        return { ...this.appInfo };
    }

    /**
     * Check if backend is ready
     */
    isBackendReady() {
        return this.backendReady;
    }

    /**
     * Show a notification (Tauri specific)
     */
    async showNotification(title, body, options = {}) {
        if (!this.appInfo.tauri) {
            console.log('🔔 Notification:', title, body);
            return;
        }

        try {
            await invoke('show_notification', {
                title,
                body,
                ...options
            });
        } catch (error) {
            console.error('Failed to show notification:', error);
        }
    }

    /**
     * Open an external URL
     */
    async openUrl(url) {
        if (!this.appInfo.tauri) {
            window.open(url, '_blank');
            return;
        }

        try {
            await invoke('open_url', { url });
        } catch (error) {
            console.error('Failed to open URL:', error);
            window.open(url, '_blank');
        }
    }

    /**
     * Show a save dialog
     */
    async showSaveDialog(defaultPath, filters = []) {
        if (!this.appInfo.tauri) {
            throw new Error('Save dialog only available in Tauri environment');
        }

        try {
            return await invoke('show_save_dialog', {
                defaultPath,
                filters
            });
        } catch (error) {
            console.error('Failed to show save dialog:', error);
            throw error;
        }
    }

    /**
     * Show an open dialog
     */
    async showOpenDialog(defaultPath, filters = [], multiple = false) {
        if (!this.appInfo.tauri) {
            throw new Error('Open dialog only available in Tauri environment');
        }

        try {
            return await invoke('show_open_dialog', {
                defaultPath,
                filters,
                multiple
            });
        } catch (error) {
            console.error('Failed to show open dialog:', error);
            throw error;
        }
    }
}

// Create singleton instance
const tauriIntegration = new TauriIntegration();

// Auto-initialize when DOM is ready
if (document.readyState === 'loading') {
    document.addEventListener('DOMContentLoaded', () => {
        tauriIntegration.initialize();
    });
} else {
    tauriIntegration.initialize();
}

// Export for use in other modules
export default tauriIntegration;

// Also provide global access
window.TauriIntegration = tauriIntegration;