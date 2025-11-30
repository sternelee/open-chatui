/**
 * Open CoreUI Tauri 集成使用示例
 *
 * 展示如何在现有前端代码中无缝使用 Tauri 集成后端
 */

import { api, initialize } from './tauri-seamless-integration.js';

/**
 * 示例 1: 基本的 API 调用（无需修改现有代码）
 */
async function basicApiCalls() {
    try {
        // 这些调用在 Tauri 和 Web 环境中都能正常工作
        // 无需任何条件判断或代码修改

        // 获取应用配置
        const config = await api.getConfig();
        console.log('应用配置:', config);

        // 获取模型列表
        const models = await api.getModels();
        console.log('可用模型:', models);

        // 健康检查
        const health = await api.healthCheck();
        console.log('后端状态:', health);

    } catch (error) {
        console.error('API 调用失败:', error);
    }
}

/**
 * 示例 2: 聊天 API 调用
 */
async function chatExample() {
    try {
        const chatPayload = {
            model: "gpt-3.5-turbo",
            messages: [
                {
                    role: "user",
                    content: "Hello, how are you?"
                }
            ],
            stream: false
        };

        // 发送聊天请求
        const response = await api.chatCompletion(chatPayload);
        console.log('聊天响应:', response);

    } catch (error) {
        console.error('聊天请求失败:', error);
    }
}

/**
 * 示例 3: 原有 fetch 代码的兼容性
 */
async function existingFetchCode() {
    // 这些原有的 fetch 代码无需修改，会自动被代理
    // 在 Tauri 环境中路由到集成后端，在 Web 环境中正常工作

    // 原有代码保持不变
    const response1 = await fetch('/api/config');
    const config = await response1.json();
    console.log('通过 fetch 获取配置:', config);

    // POST 请求也无需修改
    const response2 = await fetch('/api/chat/completions', {
        method: 'POST',
        headers: {
            'Content-Type': 'application/json',
            'Authorization': 'Bearer token-here'
        },
        body: JSON.stringify({
            model: "gpt-3.5-turbo",
            messages: [{ role: "user", content: "Test" }]
        })
    });

    const chatResponse = await response2.json();
    console.log('通过 fetch 的聊天响应:', chatResponse);
}

/**
 * 示例 4: XMLHttpRequest 兼容性
 */
function xhrExample() {
    // XMLHttpRequest 也会被自动代理
    const xhr = new XMLHttpRequest();

    xhr.onreadystatechange = function() {
        if (xhr.readyState === 4) {
            if (xhr.status === 200) {
                const response = JSON.parse(xhr.responseText);
                console.log('XHR 响应:', response);
            }
        }
    };

    xhr.open('GET', '/api/models');
    xhr.setRequestHeader('Authorization', 'Bearer token-here');
    xhr.send();
}

/**
 * 示例 5: 高级 API 客户端使用
 */
async function advancedApiUsage() {
    import { createApiClient } from './tauri-seamless-integration.js';

    // 创建专门的 API 客户端
    const userApi = createApiClient('/api/user');

    try {
        // 使用专用客户端
        const profile = await userApi.get('/profile');
        console.log('用户资料:', profile);

        const updateResult = await userApi.put('/profile', {
            name: 'New Name',
            settings: {}
        });
        console.log('更新结果:', updateResult);

    } catch (error) {
        console.error('高级 API 调用失败:', error);
    }
}

/**
 * 示例 6: 错误处理和降级
 */
async function errorHandlingExample() {
    try {
        // 正常的 API 调用
        const response = await api.get('/api/nonexistent');
        console.log('响应:', response);

    } catch (error) {
        console.error('API 错误:', error);

        // 根据环境进行不同的错误处理
        import { getStatus } from './tauri-seamless-integration.js';
        const status = getStatus();

        if (status.isTauri) {
            console.log('在 Tauri 环境中，检查集成状态...');
            console.log('集成状态:', status);
        } else {
            console.log('在 Web 环境中，检查后端服务器...');
        }
    }
}

/**
 * 示例 7: 初始化和配置
 */
async function initializationExample() {
    import { initialize, getStatus, reset } from './tauri-seamless-integration.js';

    // 手动初始化（通常不需要，会自动初始化）
    try {
        await initialize({
            debug: true, // 启用调试日志
            autoConnect: true,
            connectionTimeout: 15000,
            retryAttempts: 2
        });

        console.log('集成初始化成功');

        // 检查状态
        const status = getStatus();
        console.log('集成状态:', status);

    } catch (error) {
        console.error('初始化失败:', error);

        // 重置并重试
        reset();
        setTimeout(() => {
            initialize().catch(console.error);
        }, 2000);
    }
}

/**
 * 示例 8: 事件监听
 */
function eventListenerExample() {
    // 监听集成就绪事件
    window.addEventListener('tauri-seamless-ready', (event) => {
        console.log('Tauri 集成就绪:', event.detail);
        // 开始加载应用数据等初始化操作
        loadApplicationData();
    });

    // 监听集成错误事件
    window.addEventListener('tauri-seamless-error', (event) => {
        console.error('Tauri 集成错误:', event.detail);
        // 显示错误信息或降级到 Web 模式
        handleIntegrationError(event.detail.error);
    });
}

/**
 * 加载应用数据
 */
function loadApplicationData() {
    basicApiCalls();
    chatExample();
}

/**
 * 处理集成错误
 */
function handleIntegrationError(error) {
    console.error('集成失败，使用降级模式:', error);

    // 可以在这里实现降级逻辑
    // 例如：显示错误提示，使用外部后端等
}

/**
 * 示例 9: React/Svelte 组件中的使用
 */
function componentUsageExample() {
    // 在 React 组件中使用
    function ChatComponent() {
        const [messages, setMessages] = React.useState([]);

        const sendMessage = async (content) => {
            try {
                const response = await api.chatCompletion({
                    model: "gpt-3.5-turbo",
                    messages: [{ role: "user", content }]
                });

                setMessages(prev => [...prev, response]);
            } catch (error) {
                console.error('发送消息失败:', error);
            }
        };

        return { messages, sendMessage };
    }

    // 在 Svelte store 中使用
    function createApiStore() {
        const { subscribe, set, update } = writable({ config: null, models: [] });

        async function loadConfig() {
            const config = await api.getConfig();
            update(state => ({ ...state, config }));
        }

        async function loadModels() {
            const models = await api.getModels();
            update(state => ({ ...state, models }));
        }

        return { subscribe, loadConfig, loadModels };
    }
}

/**
 * 示例 10: WebSocket 连接
 */
function websocketExample() {
    // WebSocket 连接也会被自动代理
    const socket = io('/socket.io/');

    socket.on('connect', () => {
        console.log('WebSocket 连接已建立');
    });

    socket.on('message', (data) => {
        console.log('收到消息:', data);
    });

    // 发送消息
    socket.emit('chat', {
        message: 'Hello from Tauri integrated frontend!'
    });
}

// 导出示例函数供外部使用
export {
    basicApiCalls,
    chatExample,
    existingFetchCode,
    xhrExample,
    advancedApiUsage,
    errorHandlingExample,
    initializationExample,
    eventListenerExample,
    websocketExample,
    ChatComponent: componentUsageExample
};

// 自动运行一些示例（仅用于演示）
if (typeof window !== 'undefined') {
    // 设置事件监听
    eventListenerExample();

    // 在页面加载后运行基本示例
    if (document.readyState === 'loading') {
        document.addEventListener('DOMContentLoaded', () => {
            setTimeout(() => {
                console.log('🚀 运行 Open CoreUI Tauri 集成示例...');
                basicApiCalls();
            }, 1000);
        });
    } else {
        setTimeout(() => {
            console.log('🚀 运行 Open CoreUI Tauri 集成示例...');
            basicApiCalls();
        }, 1000);
    }
}