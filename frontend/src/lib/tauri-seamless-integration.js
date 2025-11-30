/**
 * Tauri 无缝集成模块
 *
 * 基于 Tauri HTTP Proxy 的前端集成，实现 API 的无缝迁移
 * 无需修改现有的 API 调用代码，自动将请求路由到集成后端
 */

import { initialize as initializeProxy, proxyRequest, setConfig } from './tauri-http-proxy.js';

/**
 * 检测是否在 Tauri 环境中
 */
const isTauri = typeof window !== 'undefined' && window.__TAURI__;

/**
 * 无缝集成配置
 */
const INTEGRATION_CONFIG = {
    // 是否启用自动检测
    autoDetect: true,

    // Tauri 命令名称
    commandName: 'handle_http_request',

    // API 基础路径（Web 环境下）
    webBaseUrl: 'http://localhost:8168',

    // 是否在初始化时自动连接
    autoConnect: true,

    // 连接超时时间（毫秒）
    connectionTimeout: 10000,

    // 重试次数
    retryAttempts: 3,

    // 调试模式
    debug: false
};

/**
 * 状态管理
 */
let isInitialized = false;
let isBackendReady = false;
let retryCount = 0;

/**
 * 日志工具
 */
function log(...args) {
    if (INTEGRATION_CONFIG.debug || process.env.NODE_ENV === 'development') {
        console.log('[TauriIntegration]', ...args);
    }
}

function warn(...args) {
    console.warn('[TauriIntegration]', ...args);
}

function error(...args) {
    console.error('[TauriIntegration]', ...args);
}

/**
 * 等待后端就绪
 */
async function waitForBackendReady(timeout = INTEGRATION_CONFIG.connectionTimeout) {
    return new Promise((resolve, reject) => {
        const startTime = Date.now();

        const checkReady = async () => {
            try {
                log('检查后端状态...');
                const response = await proxyRequest('/health');

                if (response.status_code === 200) {
                    isBackendReady = true;
                    log('后端已就绪');
                    resolve(true);
                    return;
                }
            } catch (err) {
                warn('后端检查失败:', err.message);
            }

            // 检查超时
            if (Date.now() - startTime > timeout) {
                reject(new Error('后端启动超时'));
                return;
            }

            // 继续等待
            setTimeout(checkReady, 1000);
        };

        checkReady();
    });
}

/**
 * 测试 API 连接
 */
async function testApiConnection() {
    try {
        log('测试 API 连接...');

        // 测试配置接口
        const configResponse = await proxyRequest('/api/config');
        if (configResponse.status_code !== 200) {
            throw new Error('配置接口测试失败');
        }

        // 测试模型接口
        const modelsResponse = await proxyRequest('/api/models');
        if (modelsResponse.status_code !== 200) {
            throw new Error('模型接口测试失败');
        }

        log('API 连接测试通过');
        return true;
    } catch (err) {
        error('API 连接测试失败:', err);
        return false;
    }
}

/**
 * 初始化无缝集成
 */
export async function initialize(options = {}) {
    if (isInitialized) {
        log('已经初始化，跳过');
        return true;
    }

    // 合并配置
    Object.assign(INTEGRATION_CONFIG, options);
    setConfig({
        tauriCommand: INTEGRATION_CONFIG.commandName,
        debug: INTEGRATION_CONFIG.debug
    });

    if (!isTauri) {
        log('非 Tauri 环境，跳过集成');
        return true;
    }

    log('初始化 Tauri 无缝集成...');
    log('环境:', isTauri ? 'Tauri Desktop' : 'Web Browser');
    log('命令:', INTEGRATION_CONFIG.commandName);

    try {
        // 初始化 HTTP 代理
        initializeProxy('/');

        if (INTEGRATION_CONFIG.autoConnect) {
            log('等待后端启动...');
            await waitForBackendReady();

            log('测试 API 连接...');
            const apiTest = await testApiConnection();

            if (!apiTest && retryCount < INTEGRATION_CONFIG.retryAttempts) {
                retryCount++;
                log(`重试 API 连接 (${retryCount}/${INTEGRATION_CONFIG.retryAttempts})`);
                await new Promise(resolve => setTimeout(resolve, 2000));
                return await initialize(options);
            }
        }

        isInitialized = true;
        log('✅ Tauri 无缝集成初始化完成');

        // 发出就绪事件
        window.dispatchEvent(new CustomEvent('tauri-seamless-ready', {
            detail: {
                environment: 'tauri',
                backendReady: isBackendReady,
                retryCount
            }
        }));

        return true;

    } catch (err) {
        error('❌ Tauri 无缝集成初始化失败:', err);
        isInitialized = false;

        // 发出错误事件
        window.dispatchEvent(new CustomEvent('tauri-seamless-error', {
            detail: { error: err.message }
        }));

        throw err;
    }
}

/**
 * 手动发送 API 请求
 */
export async function apiRequest(path, options = {}) {
    if (!isInitialized) {
        await initialize();
    }

    try {
        const response = await proxyRequest(path, {
            method: options.method || 'GET',
            headers: options.headers || {},
            body: options.body
        });

        // 解析响应体
        let body;
        try {
            const decoder = new TextDecoder('utf-8');
            const bodyText = decoder.decode(new Uint8Array(response.body));
            body = JSON.parse(bodyText);
        } catch {
            body = response.body;
        }

        return {
            ok: response.status_code >= 200 && response.status_code < 300,
            status: response.status_code,
            statusText: response.status_code.toString(),
            headers: response.headers,
            data: body
        };

    } catch (err) {
        error('API 请求失败:', path, err);
        throw err;
    }
}

/**
 * API 方法快捷调用
 */
export const api = {
    /**
     * GET 请求
     */
    async get(path, headers = {}) {
        return apiRequest(path, { method: 'GET', headers });
    },

    /**
     * POST 请求
     */
    async post(path, data, headers = {}) {
        return apiRequest(path, {
            method: 'POST',
            headers: {
                'Content-Type': 'application/json',
                ...headers
            },
            body: JSON.stringify(data)
        });
    },

    /**
     * PUT 请求
     */
    async put(path, data, headers = {}) {
        return apiRequest(path, {
            method: 'PUT',
            headers: {
                'Content-Type': 'application/json',
                ...headers
            },
            body: JSON.stringify(data)
        });
    },

    /**
     * DELETE 请求
     */
    async delete(path, headers = {}) {
        return apiRequest(path, { method: 'DELETE', headers });
    },

    /**
     * 获取配置
     */
    async getConfig() {
        const response = await this.get('/api/config');
        return response.data;
    },

    /**
     * 获取模型列表
     */
    async getModels() {
        const response = await this.get('/api/models');
        return response.data;
    },

    /**
     * 聊天完成
     */
    async chatCompletion(payload) {
        const response = await this.post('/api/chat/completions', payload);
        return response.data;
    },

    /**
     * 健康检查
     */
    async healthCheck() {
        const response = await this.get('/health');
        return response.data;
    }
};

/**
 * 创建 API 客户端实例
 */
export function createApiClient(basePath = '') {
    return {
        async request(path, options = {}) {
            return apiRequest(basePath + path, options);
        },

        async get(path, headers = {}) {
            return api.get(basePath + path, headers);
        },

        async post(path, data, headers = {}) {
            return api.post(basePath + path, data, headers);
        },

        async put(path, data, headers = {}) {
            return api.put(basePath + path, data, headers);
        },

        async delete(path, headers = {}) {
            return api.delete(basePath + path, headers);
        }
    };
}

/**
 * 获取集成状态
 */
export function getStatus() {
    return {
        initialized: isInitialized,
        backendReady: isBackendReady,
        isTauri,
        environment: isTauri ? 'tauri' : 'web',
        retryCount
    };
}

/**
 * 重置集成状态
 */
export function reset() {
    isInitialized = false;
    isBackendReady = false;
    retryCount = 0;
    log('集成状态已重置');
}

/**
 * 自动初始化（如果在 Tauri 环境中）
 */
if (isTauri && INTEGRATION_CONFIG.autoDetect) {
    // 等待 DOM 加载完成
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => {
            initialize().catch(err => {
                error('自动初始化失败:', err);
            });
        });
    } else {
        initialize().catch(err => {
            error('自动初始化失败:', err);
        });
    }
}

// 默认导出
export default {
    initialize,
    api,
    apiRequest,
    createApiClient,
    getStatus,
    reset
};

// 全局访问
window.TauriSeamlessIntegration = {
    initialize,
    api,
    getStatus,
    reset
};