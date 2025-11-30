# Tauri 集成后端快速开始指南

本指南帮助您快速将 Open CoreUI 的 sidecar 架构迁移到集成的 Tauri 后端架构。

## 🚀 快速开始

### 1. 导入代理模块

在您的前端应用入口文件（如 `index.html` 或主 JS 文件）中导入代理模块：

```html
<!DOCTYPE html>
<html lang="zh-CN">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Open CoreUI</title>

    <!-- 导入 Tauri HTTP 代理 -->
    <script type="module">
        import './src/lib/tauri-http-proxy.js';
        import './src/lib/tauri-seamless-integration.js';
        import './src/lib/usage-example.js';
    </script>
</head>
<body>
    <!-- 您的应用内容 -->
    <div id="app"></div>
</body>
</html>
```

### 2. 现有代码无需修改

您的现有 API 调用代码无需任何修改：

```javascript
// ✅ 这些代码无需修改，会自动工作
fetch('/api/config')
    .then(response => response.json())
    .then(config => console.log(config));

// ✅ POST 请求也无需修改
fetch('/api/chat/completions', {
    method: 'POST',
    headers: {
        'Content-Type': 'application/json'
    },
    body: JSON.stringify({
        model: "gpt-3.5-turbo",
        messages: [{ role: "user", content: "Hello" }]
    })
});
```

### 3. 构建和运行

```bash
# 构建集成版本
make build-integrated

# 开发模式运行
make run-desktop
```

## 🔧 高级配置

### 自定义代理配置

```javascript
import { initialize } from './src/lib/tauri-seamless-integration.js';

// 自定义初始化选项
await initialize({
    debug: true,                    // 启用调试日志
    autoConnect: true,              // 自动连接后端
    connectionTimeout: 15000,       // 连接超时 (ms)
    retryAttempts: 3,              // 重试次数
    commandName: 'handle_http_request'  // Tauri 命令名称
});
```

### 手动 API 调用

```javascript
import { api } from './src/lib/tauri-seamless-integration.js';

// 使用便捷方法
const config = await api.getConfig();
const models = await api.getModels();
const response = await api.chatCompletion(payload);

// 或使用通用方法
const response = await api.post('/api/custom', data);
```

### 创建专门的 API 客户端

```javascript
import { createApiClient } from './src/lib/tauri-seamless-integration.js';

// 用户 API 客户端
const userApi = createApiClient('/api/user');
const profile = await userApi.get('/profile');

// 自定义基础路径
const customApi = createApiClient('/v1/custom');
const result = await customApi.post('/endpoint', data);
```

## 🛠️ 环境检测

系统会自动检测运行环境并适配：

```javascript
import { getStatus } from './src/lib/tauri-seamless-integration.js';

const status = getStatus();
console.log(status);
// {
//   initialized: true,
//   backendReady: true,
//   isTauri: true,
//   environment: 'tauri',
//   retryCount: 0
// }
```

## 🎯 框架集成

### React 示例

```jsx
import React, { useState, useEffect } from 'react';
import { api } from './src/lib/tauri-seamless-integration.js';

function ChatComponent() {
    const [messages, setMessages] = useState([]);
    const [models, setModels] = useState([]);

    useEffect(() => {
        // 加载模型列表
        api.getModels().then(data => setModels(data.data || []));
    }, []);

    const sendMessage = async (content) => {
        try {
            const response = await api.chatCompletion({
                model: models[0]?.id || 'gpt-3.5-turbo',
                messages: [{ role: 'user', content }]
            });

            setMessages(prev => [...prev, response]);
        } catch (error) {
            console.error('发送失败:', error);
        }
    };

    return (
        <div>
            {/* 您的聊天界面 */}
        </div>
    );
}
```

### Svelte 示例

```javascript
import { onMount } from 'svelte';
import { api } from './src/lib/tauri-seamless-integration.js';
import { writable } from 'svelte/store';

export function createAppStore() {
    const config = writable(null);
    const models = writable([]);
    const loading = writable(true);

    onMount(async () => {
        try {
            const [configData, modelsData] = await Promise.all([
                api.getConfig(),
                api.getModels()
            ]);

            config.set(configData);
            models.set(modelsData.data || []);
        } catch (error) {
            console.error('加载失败:', error);
        } finally {
            loading.set(false);
        }
    });

    return { config, models, loading };
}
```

### Vue 示例

```vue
<template>
    <div>
        <div v-if="loading">加载中...</div>
        <div v-else>
            <h1>{{ config.name }}</h1>
            <select v-model="selectedModel">
                <option v-for="model in models" :key="model.id" :value="model.id">
                    {{ model.name }}
                </option>
            </select>
        </div>
    </div>
</template>

<script>
import { ref, onMounted } from 'vue';
import { api } from './src/lib/tauri-seamless-integration.js';

export default {
    setup() {
        const config = ref({});
        const models = ref([]);
        const loading = ref(true);
        const selectedModel = ref('');

        onMounted(async () => {
            try {
                const [configData, modelsData] = await Promise.all([
                    api.getConfig(),
                    api.getModels()
                ]);

                config.value = configData;
                models.value = modelsData.data || [];

                if (models.value.length > 0) {
                    selectedModel.value = models.value[0].id;
                }
            } catch (error) {
                console.error('初始化失败:', error);
            } finally {
                loading.value = false;
            }
        });

        return {
            config,
            models,
            loading,
            selectedModel
        };
    }
};
</script>
```

## 🔍 调试和故障排除

### 启用调试模式

```javascript
import { initialize } from './src/lib/tauri-seamless-integration.js';

await initialize({
    debug: true  // 显示详细日志
});
```

### 常见问题

1. **后端连接失败**
   ```javascript
   // 检查后端状态
   const status = getStatus();
   console.log('后端就绪:', status.backendReady);
   ```

2. **API 调用失败**
   ```javascript
   // 检查错误详情
   try {
       await api.get('/api/test');
   } catch (error) {
       console.error('API 错误:', error.message);

       // 根据环境处理错误
       if (status.isTauri) {
           console.log('Tauri 环境错误处理');
       } else {
           console.log('Web 环境错误处理');
       }
   }
   ```

3. **WebSocket 连接问题**
   ```javascript
   // WebSocket 也会被自动代理
   const socket = io('/socket.io/');

   socket.on('connect_error', (error) => {
       console.error('WebSocket 连接失败:', error);
   });
   ```

## 📦 部署

### Tauri 桌面应用

```bash
# 生产构建
make build-integrated

# 检查输出
ls -la src-tauri/target/release/bundle/
```

### Web 应用

您的代码无需修改即可在 Web 环境中运行，代理会自动禁用：

```javascript
// 这段代码在 Tauri 和 Web 环境中都能工作
fetch('/api/config').then(r => r.json());
```

## 🎉 完成！

恭喜！您已经成功集成了 Tauri 后端代理。现在您的应用：

- ✅ **无需修改现有 API 代码**
- ✅ **自动检测运行环境**
- ✅ **支持所有 HTTP 方法**
- ✅ **支持 WebSocket 和 Socket.IO**
- ✅ **提供降级处理**
- ✅ **包含完整错误处理**

享受更快的启动速度、更低的资源占用和更好的用户体验！

## 📚 更多资源

- [完整迁移指南](./MIGRATION_GUIDE.md)
- [API 文档](./frontend/src/lib/tauri-seamless-integration.js)
- [使用示例](./frontend/src/lib/usage-example.js)
- [故障排除](./docs/troubleshooting.md)