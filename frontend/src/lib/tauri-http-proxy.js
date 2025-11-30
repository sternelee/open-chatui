/**
 * Tauri HTTP Proxy for Open CoreUI
 *
 * 基于 Tauri Axum HTMX 模式的前端 HTTP 请求代理
 * 实现前端到集成的 Tauri 后端的无缝 API 迁移
 */

// 检查是否在 Tauri 环境中
const isTauri = typeof window !== 'undefined' && window.__TAURI__;

// 配置
const CONFIG = {
    // Tauri 命令名称
    tauriCommand: 'handle_http_request',

    // 需要代理的路径前缀
    proxyPaths: [
        '/api',
        '/openai',
        '/ws',
        '/health',
        '/socket.io'
    ],

    // 排除代理的路径（静态资源等）
    excludePaths: [
        '/static',
        '/assets',
        '/favicon',
        '/manifest.json',
        'data:',
        'blob:',
        'ipc://',
        'http://',
        'https://'
    ],

    // 是否启用详细日志
    debug: false
};

/**
 * 日志工具
 */
function log(...args) {
    if (CONFIG.debug || process.env.NODE_ENV === 'development') {
        console.log('[TauriHTTPProxy]', ...args);
    }
}

function error(...args) {
    console.error('[TauriHTTPProxy]', ...args);
}

/**
 * 检查 URL 是否需要代理
 */
function shouldProxy(url) {
    try {
        // 排除特殊协议
        for (const prefix of CONFIG.excludePaths) {
            if (url.startsWith(prefix)) {
                return false;
            }
        }

        // 检查是否为相对路径且匹配代理路径
        if (url.startsWith('/')) {
            for (const proxyPath of CONFIG.proxyPaths) {
                if (url.startsWith(proxyPath)) {
                    return true;
                }
            }
        }

        return false;
    } catch (e) {
        error('URL 解析错误:', url, e);
        return false;
    }
}

/**
 * 处理 HTTP 重定向
 */
async function handleRedirect(response) {
    const statusCode = parseInt(response.status_code);

    // HTTP 重定向状态码
    if ([301, 302, 303, 307, 308].includes(statusCode)) {
        const location = response.headers?.location;

        if (location) {
            log('处理重定向:', location);

            const redirectRequest = {
                uri: location,
                method: 'GET', // 重定向通常使用 GET
                headers: {},
                body: null
            };

            return await invoke(CONFIG.tauriCommand, {
                localRequest: redirectRequest
            });
        }
    }

    return response;
}

/**
 * 代理 Fetch API
 */
function proxyFetch() {
    if (!isTauri) {
        log('非 Tauri 环境，跳过 Fetch 代理');
        return;
    }

    const { invoke } = window.__TAURI__.core;
    const originalFetch = window.fetch;

    window.fetch = async function (...args) {
        const [url, options = {}] = args;

        // 跳过特殊 URL
        if (!shouldProxy(url)) {
            log('跳过代理:', url);
            return originalFetch(...args);
        }

        log('代理请求:', url, options.method || 'GET');

        try {
            // 构建请求对象
            const request = {
                uri: url,
                method: options.method || 'GET',
                headers: options.headers || {},
                body: options.body
            };

            // 通过 Tauri 调用后端
            let response = await invoke(CONFIG.tauriCommand, {
                localRequest: request
            });

            // 处理重定向
            response = await handleRedirect(response);

            // 转换响应体
            const bodyByteArray = new Uint8Array(response.body);
            const decoder = new TextDecoder('utf-8');
            const bodyText = decoder.decode(bodyByteArray);

            const status = parseInt(response.status_code);
            const headers = new Headers(response.headers);

            log('代理响应:', status, url);

            return new Response(bodyText, {
                status,
                headers,
                url: url // 保持原始 URL
            });

        } catch (err) {
            error('代理请求失败:', url, err);

            // 降级到原始 fetch
            log('降级到原始 fetch:', url);
            return originalFetch(...args);
        }
    };

    // 保存原始 fetch 以供特殊用途
    window.originalFetch = originalFetch;
    log('Fetch 代理已启用');
}

/**
 * EventTarget 实现（用于 XMLHttpRequest）
 */
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
            this.eventListeners[event].forEach(callback => {
                try {
                    callback.apply(this, args);
                } catch (e) {
                    error('事件回调错误:', event, e);
                }
            });
        }
    }
}

/**
 * 代理 XMLHttpRequest
 */
function proxyXMLHttpRequest() {
    if (!isTauri) {
        log('非 Tauri 环境，跳过 XMLHttpRequest 代理');
        return;
    }

    const originalXMLHttpRequest = window.XMLHttpRequest;

    class ProxyXMLHttpRequest extends EventTarget {
        constructor() {
            super();

            // 标准属性
            this.onload = null;
            this.onerror = null;
            this.onreadystatechange = null;
            this.onloadstart = null;
            this.onloadend = null;
            this.onprogress = null;
            this.ontimeout = null;
            this.onabort = null;

            // 状态属性
            this.readyState = 0;
            this.status = 0;
            this.statusText = '';
            this.response = null;
            this.responseText = null;
            this.responseXML = null;
            this.responseType = '';
            this.responseURL = '';

            // 请求属性
            this.method = null;
            this.url = null;
            this.async = true;
            this.user = null;
            this.password = null;

            // 内部状态
            this.requestHeaders = {};
            this.responseHeaders = {};
            this.controller = new AbortController();
            this.upload = new EventTarget();
            this.timeout = 0;
        }

        open(method, url, async = true, user = null, password = null) {
            this.method = method.toUpperCase();
            this.url = url;
            this.async = async;
            this.user = user;
            this.password = password;
            this.readyState = 1;
            this._triggerEvent('readystatechange');
            log('XHR 打开:', method, url);
        }

        send(data = null) {
            if (!this.shouldProxy()) {
                return this._sendNative(data);
            }

            log('XHR 代理请求:', this.method, this.url);

            const options = {
                method: this.method,
                headers: { ...this.requestHeaders },
                body: data,
                signal: this.controller.signal,
                mode: 'cors',
                credentials: (this.user && this.password) ? 'include' : 'same-origin',
            };

            // 添加认证头
            if (this.user && this.password) {
                const base64Credentials = btoa(`${this.user}:${this.password}`);
                options.headers['Authorization'] = `Basic ${base64Credentials}`;
            }

            this.readyState = 2;
            this._triggerEvent('readystatechange');

            // 使用 fetch 发送请求
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
                    this.responseText = typeof responseData === 'string' ?
                        responseData :
                        JSON.stringify(responseData);

                    this._triggerEvent('readystatechange');

                    if (this.onload) {
                        this.onload();
                    }

                    log('XHR 代理响应:', this.status, this.url);
                })
                .catch(error => {
                    this.readyState = 4;
                    error('XHR 代理错误:', error);

                    if (this.onerror) {
                        this.onerror(error);
                    }
                });
        }

        _sendNative(data) {
            log('XHR 原始请求:', this.method, this.url);

            const xhr = new originalXMLHttpRequest();

            // 复制事件监听器
            ['load', 'error', 'readystatechange', 'progress', 'timeout', 'abort', 'loadstart', 'loadend'].forEach(event => {
                if (this[`on${event}`]) {
                    xhr[`on${event}`] = this[`on${event}`];
                }
            });

            xhr.open(this.method, this.url, this.async, this.user, this.password);

            // 复制请求头
            Object.entries(this.requestHeaders).forEach(([key, value]) => {
                xhr.setRequestHeader(key, value);
            });

            xhr.timeout = this.timeout;
            xhr.responseType = this.responseType;

            xhr.onreadystatechange = () => {
                this.readyState = xhr.readyState;
                this.status = xhr.status;
                this.statusText = xhr.statusText;
                this.responseText = xhr.responseText;
                this.response = xhr.response;
                this.responseHeaders = this._parseNativeHeaders(xhr.getAllResponseHeaders());

                if (this.onreadystatechange) {
                    this.onreadystatechange();
                }
            };

            xhr.send(data);
        }

        shouldProxy() {
            return isTauri && shouldProxy(this.url);
        }

        setRequestHeader(header, value) {
            this.requestHeaders[header] = value;
        }

        getResponseHeader(header) {
            return this.responseHeaders[header.toLowerCase()] || null;
        }

        getAllResponseHeaders() {
            return Object.entries(this.responseHeaders)
                .map(([key, value]) => `${key}: ${value}`)
                .join('\r\n');
        }

        abort() {
            this.controller.abort();
            this.readyState = 0;
            this._triggerEvent('abort');
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

        _parseNativeHeaders(headersString) {
            const headers = {};
            if (headersString) {
                headersString.split('\r\n').forEach(line => {
                    const [key, ...valueParts] = line.split(':');
                    if (key && valueParts.length) {
                        headers[key.toLowerCase().trim()] = valueParts.join(':').trim();
                    }
                });
            }
            return headers;
        }

        _parseResponse(response) {
            const contentType = response.headers.get('content-type');

            switch (this.responseType) {
                case 'json':
                    return response.json();
                case 'blob':
                    return response.blob();
                case 'arraybuffer':
                    return response.arrayBuffer();
                case 'document':
                    return response.text().then(text => {
                        const parser = new DOMParser();
                        return parser.parseFromString(text, 'text/html');
                    });
                default:
                    if (contentType && contentType.includes('application/json')) {
                        return response.json();
                    } else if (contentType && (contentType.includes('text/') || contentType.includes('xml'))) {
                        return response.text();
                    } else {
                        return response.blob();
                    }
            }
        }
    }

    window.XMLHttpRequest = ProxyXMLHttpRequest;
    log('XMLHttpRequest 代理已启用');
}

/**
 * 初始化代理
 */
export function initialize(initialPath = '/', options = {}) {
    // 合并配置
    Object.assign(CONFIG, options);

    log('初始化 Tauri HTTP 代理');
    log('代理路径:', CONFIG.proxyPaths);
    log('排除路径:', CONFIG.excludePaths);

    if (!isTauri) {
        log('非 Tauri 环境，代理未启用');
        return;
    }

    // 启用代理
    proxyFetch();
    proxyXMLHttpRequest();

    // 加载初始页面
    if (initialPath && initialPath !== '/') {
        window.addEventListener('DOMContentLoaded', async () => {
            try {
                log('加载初始页面:', initialPath);
                const response = await window.fetch(initialPath);
                const html = await response.text();

                if (response.ok) {
                    document.documentElement.innerHTML = html;

                    // 如果存在 htmx，处理它
                    if (window.htmx) {
                        window.htmx.process(document.documentElement);
                    }

                    log('初始页面加载完成');
                } else {
                    error('初始页面加载失败:', response.status);
                }
            } catch (err) {
                error('初始页面加载错误:', err);
            }
        });
    }

    log('Tauri HTTP 代理初始化完成');
}

/**
 * 手动调用代理
 */
export async function proxyRequest(url, options = {}) {
    if (!isTauri) {
        throw new Error('非 Tauri 环境');
    }

    const { invoke } = window.__TAURI__.core;

    const request = {
        uri: url,
        method: options.method || 'GET',
        headers: options.headers || {},
        body: options.body
    };

    let response = await invoke(CONFIG.tauriCommand, {
        localRequest: request
    });

    // 处理重定向
    response = await handleRedirect(response);

    return response;
}

/**
 * 获取配置
 */
export function getConfig() {
    return { ...CONFIG };
}

/**
 * 设置配置
 */
export function setConfig(newConfig) {
    Object.assign(CONFIG, newConfig);
}

// 自动初始化
if (isTauri && window.location.pathname === '/') {
    initialize();
}

// 导出供外部使用
export default {
    initialize,
    proxyRequest,
    getConfig,
    setConfig
};