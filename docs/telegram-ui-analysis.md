# Telegram UI 復刻實作策略分析

> 分析時間: 2026-01-31  
> 分析範圍: Vue 3 前端 Telegram UI 復刻  
> 研究深度: 多代理平行搜索 + 官方文件 + 開源實現

---

## 📊 現況總結

### 當前專案狀態
- **技術堆疊**: Vue 3 + TypeScript + Vite + Tailwind CSS + Pinia
- **現有視圖**: 
  - `HomeView.vue` - 首頁，有加入遊戲按鈕
  - `GameView.vue` - 遊戲房間（目前是 placeholder）
- **狀態管理**: Pinia store (`game.ts`) 已設置 WebSocket 連接
- **現有 UI**: 極簡風格，使用 Tailwind 基本樣式

---

## 🔍 Telegram UI 研究發現

### 1. 官方顏色系統 (Mini Apps)

根據 [Telegram Mini Apps 官方文件](https://docs.telegram-mini-apps.com/platform/theming)：

**Dark Mode 顏色值**:
```json
{
  "bg_color": "#17212b",
  "secondary_bg_color": "#232e3c", 
  "text_color": "#f5f5f5",
  "hint_color": "#708499",
  "link_color": "#6ab3f3",
  "button_color": "#5288c1",
  "button_text_color": "#ffffff",
  "accent_text_color": "#6ab2f2",
  "section_bg_color": "#17212b",
  "header_bg_color": "#17212b",
  "subtitle_text_color": "#708499",
  "destructive_text_color": "#ec3942",
  "bottom_bar_bg_color": "#ffffff"
}
```

**Light Mode 顏色值**:
```json
{
  "bg_color": "#ffffff",
  "secondary_bg_color": "#efeff3",
  "text_color": "#000000",
  "hint_color": "#999999",
  "link_color": "#2481cc",
  "button_color": "#2481cc",
  "button_text_color": "#ffffff"
}
```

**訊息氣泡專用顏色** (從 Telegram Web K 原始碼分析):
- **自己發送**: `#2b5278` (深藍)
- **他人發送**: `#182533` (深灰)
- **背景**: `#0e1621` (聊天區)

---

### 2. 可用的 Vue 函式庫

#### A. TeleVue (@erfanmola) ⭐ 推薦
- **GitHub**: https://github.com/erfanmola/TeleVue
- **文件**: https://erfanmola.github.io/TeleVue/
- **元件**: Switch, Checkbox, RadioButton, Section, List, ColorPicker, Chips, Avatar, Skeleton, Toast, Tabs, BackButton, MainButton
- **用途**: Telegram Web Apps 專用 UI 庫

#### B. vue-advanced-chat
- **GitHub**: https://github.com/advanced-chat/vue-advanced-chat
- **Stars**: 2,000+
- **特色**: 完整的聊天室解決方案，支援多種框架

#### C. VantChatUI
- **GitHub**: https://github.com/keepingFE/VantChatUI
- **特色**: Vue 3 + Vant 4 + Tailwind，20+ 聊天元件

#### D. vue-telegram
- **GitHub**: https://github.com/deptyped/vue-telegram
- **Stars**: 267
- **用途**: Telegram SDK 整合

---

### 3. 參考實現 (Telegram Web K)

**官方開源客戶端**: https://github.com/TelegramOrg/Telegram-web-k

**關鍵檔案**:
- `/src/scss/variables.scss` - 設計 token (邊框圓角、間距)
- `/src/scss/partials/_chatBubble.scss` - 訊息氣泡樣式
- `/src/scss/partials/_leftSidebar.scss` - 側邊欄 (80px 收合 / 420px 展開)
- `/src/scss/partials/_chat.scss` - 聊天區域佈局
- `/src/components/chat/bubbles.ts` - 氣泡渲染邏輯

**佈局數值**:
```scss
// 側邊欄
$left-sidebar-width: 80px;        // 收合
$left-sidebar-max-width: 420px;   // 展開 (1680px 螢幕 / 4)

// 聊天區
$messages-container-width: 728px;
$chat-input-size: 3.375rem;       // Desktop
$chat-padding: 8px;

// 圓角
$border-radius-medium: 5px;
$border-radius-big: 15px;
```

---

### 4. CSS 變數模式

**Telegram Mini Apps 官方變數**:
```css
:root {
  --tg-theme-bg-color: #17212b;
  --tg-theme-text-color: #f5f5f5;
  --tg-theme-hint-color: #708499;
  --tg-theme-link-color: #6ab3f3;
  --tg-theme-button-color: #5288c1;
  --tg-theme-secondary-bg-color: #232e3c;
  --tg-viewport-height: 100vh;
  --tg-color-scheme: dark;
}
```

---

### 5. Telegram Clone 專案參考

#### TibebeJS/telegram-desktop-UI-clone
- **Tech**: Vue + TailwindCSS
- **元件**: Sidebar, ChatList, Chat, TextMessage
- **特色**: 完整三欄佈局實現

#### jensendarren/telegram-clone  
- **Tech**: VueJS + TypeScript + Vuetify
- **描述**: Telegram Web UI 克隆

---

### 6. 訊息氣泡實現模式

**從多個開源專案分析**:

```vue
<!-- 基本結構 -->
<div class="message-container" :class="{ 'outgoing': isOwnMessage }">
  <div class="bubble">
    <div class="bubble-content">{{ message.text }}</div>
    <div class="bubble-meta">
      <span class="timestamp">{{ time }}</span>
    </div>
  </div>
</div>
```

**CSS 樣式**:
```css
.message-container {
  display: flex;
  padding: 2px 8px;
}

.message-container.outgoing {
  justify-content: flex-end;
}

.bubble {
  max-width: 70%;
  padding: 12px;
  border-radius: 12px;
  position: relative;
}

.bubble::after {
  content: " ";
  position: absolute;
  width: 0;
  height: 0;
  bottom: 0;
  left: -6px;
  border-bottom: 25px solid white;
  border-left: 20px solid transparent;
}

.outgoing .bubble {
  background: #2b5278;
  border-bottom-right-radius: 5px;
}

.incoming .bubble {
  background: #182533;
  border-bottom-left-radius: 5px;
}
```

---

### 7. 字體系統

**平台對應**:
- **iOS**: San Francisco (SF Pro)
- **Android**: Roboto
- **Web**: 系統字體堆疊

**建議字體堆疊**:
```css
font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
```

**注意**: Telegram 沒有官方字級系統，使用系統預設。

---

### 8. Figma UI Kit 資源

1. **Telegram iOS UI Kit**: https://www.figma.com/community/file/1342906900898425723
2. **Telegram Design System**: https://www.figma.com/community/file/9884669276242179947
3. **Telegram Mini Apps UI Kit**: https://www.figma.com/community/file/1348989725141777736
4. **Telegram App UI Kit (310 Screens)**: https://www.titanui.com/129656-telegram-ui-kit-310-screens/

---

## 🎯 使用者決策確認

基於訪談，已確認以下需求：

| 決策項目 | 選擇 | 說明 |
|---------|------|------|
| **實作方案** | 混合方案 (C) | TeleVue 基礎元件 + 手刻聊天元件 |
| **主題支援** | Light/Dark 雙主題 | 完整主題切換 |
| **手機佈局** | 側邊欄隱藏 | 手機版像 Telegram App |

---

## 🏗️ 建議實作策略

### 方案 C: 混合方案 (已確認)

**策略**:
- ✅ 使用 TeleVue 的基礎元件 (Button, Switch, Avatar 等)
- ✅ 手刻核心聊天元件 (MessageBubble, ChatInput, SymbolKeyboard)
- ✅ 實作 Light/Dark 雙主題系統
- ✅ 響應式設計 (手機側邊欄隱藏)

**實作步驟**:
1. 安裝 TeleVue: `bun add @erfanmola/televue`
2. 擴展 Tailwind 配置加入 Telegram 顏色
3. 建立主題系統 (CSS 變數)
4. 實作佈局元件 (TelegramLayout)
5. 實作聊天元件
6. 整合符號輸入系統

---

## 🧩 需要建立的元件清單

### Phase 1: 基礎佈局
1. **TelegramLayout** - 雙欄佈局 (Sidebar + ChatArea)
   - Props: `isMobile`, `sidebarVisible`
   - 響應式: 手機隱藏側邊欄

2. **Sidebar** - 左側聊天列表
   - 使用 TeleVue List + Avatar
   - 顯示 4 位玩家

3. **ChatArea** - 右側聊天區域
   - 包含 ChatHeader, MessageList, ChatInput

### Phase 2: 聊天元件
4. **MessageBubble** - 訊息氣泡
   - Props: `message`, `isOwn`, `isCensored`
   - 樣式: 自己 `#2b5278`，他人 `#182533`
   - 支援審查遮罩顯示

5. **MessageList** - 訊息列表
   - 自動滾動到底部
   - 分組顯示

6. **ChatInput** - 輸入區域
   - 符號輸入鍵盤整合
   - 語音輸入按鈕
   - 發送按鈕

7. **ChatHeader** - 聊天室標題欄
   - 國家名稱
   - 連線狀態指示器

### Phase 3: 符號系統
8. **SymbolKeyboard** - 符號輸入鍵盤
   - 4x7 網格 (28格，2格空白或功能鍵)
   - 點擊輸入

9. **SymbolDisplay** - 符號顯示
   - 大圖示顯示已選符號

### Phase 4: 輔助元件
10. **CensoredText** - 審查遮罩文字
    - 敏感詞顯示為 ****
    - 支援懸停顯示 (選配)

11. **ConnectionStatus** - 連線狀態指示器
12. **ThemeToggle** - 主題切換開關 (使用 TeleVue Switch)

---

## 🎨 Telegram 風格規範

### 顏色系統 (Tailwind 擴展)

```javascript
// tailwind.config.js
module.exports = {
  theme: {
    extend: {
      colors: {
        telegram: {
          // 基礎背景
          'bg': '#17212b',
          'bg-light': '#ffffff',
          'bg-secondary': '#232e3c',
          'bg-secondary-light': '#efeff3',
          'bg-chat': '#0e1621',
          
          // 訊息氣泡
          'message-out': '#2b5278',
          'message-in': '#182533',
          'message-out-light': '#dcf8c6',
          'message-in-light': '#ffffff',
          
          // 強調色
          'accent': '#5288c1',
          'accent-light': '#6ab3f3',
          'button': '#5288c1',
          'button-light': '#2481cc',
          
          // 文字
          'text': '#f5f5f5',
          'text-light': '#000000',
          'text-secondary': '#708499',
          'text-secondary-light': '#999999',
          
          // 其他
          'link': '#6ab3f3',
          'link-light': '#2481cc',
          'destructive': '#ec3942',
        }
      }
    }
  }
}
```

### CSS 變數方案 (主題切換)

```css
/* styles/telegram-theme.css */
:root {
  /* Dark Mode (預設) */
  --tg-bg: #17212b;
  --tg-bg-secondary: #232e3c;
  --tg-text: #f5f5f5;
  --tg-text-secondary: #708499;
  --tg-message-out: #2b5278;
  --tg-message-in: #182533;
  --tg-accent: #5288c1;
  --tg-link: #6ab3f3;
}

[data-theme="light"] {
  --tg-bg: #ffffff;
  --tg-bg-secondary: #efeff3;
  --tg-text: #000000;
  --tg-text-secondary: #999999;
  --tg-message-out: #dcf8c6;
  --tg-message-in: #ffffff;
  --tg-accent: #2481cc;
  --tg-link: #2481cc;
}
```

### 佈局規範
- **側邊欄寬度**: 320px (Desktop) / 0px (Mobile，可滑出)
- **聊天區**: 剩餘寬度 (flex: 1)
- **訊息氣泡最大寬度**: 70%
- **圓角**: 12px (氣泡), 8px (按鈕), 16px (卡片)
- **基礎間距**: 8px

### 字體規範
```css
font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Helvetica, Arial, sans-serif;
```

---

## 📁 建議檔案結構

```
frontend/src/
├── components/
│   ├── layout/
│   │   ├── TelegramLayout.vue      # 雙欄佈局
│   │   ├── Sidebar.vue             # 側邊欄
│   │   └── ChatArea.vue            # 聊天區域
│   ├── chat/
│   │   ├── MessageBubble.vue       # 訊息氣泡
│   │   ├── MessageList.vue         # 訊息列表
│   │   ├── ChatInput.vue           # 輸入區
│   │   └── ChatHeader.vue          # 標題欄
│   ├── symbols/
│   │   ├── SymbolKeyboard.vue      # 符號鍵盤
│   │   └── SymbolDisplay.vue       # 符號顯示
│   └── common/
│       ├── CensoredText.vue        # 審查遮罩
│       ├── ConnectionStatus.vue    # 連線狀態
│       └── ThemeToggle.vue         # 主題切換
├── composables/
│   └── useTheme.ts                 # 主題管理
├── styles/
│   └── telegram-theme.css          # Telegram 主題變數
├── views/
│   └── GameView.vue                # 重構後的遊戲視圖
└── stores/
    └── game.ts                     # 現有 (需擴展)
```

---

## 📚 參考資源

### 官方資源
1. **TeleVue 文件**: https://erfanmola.github.io/TeleVue/
2. **Telegram Mini Apps 文件**: https://docs.telegram-mini-apps.com/
3. **Telegram Web K**: https://github.com/TelegramOrg/Telegram-web-k

### 開源實現
4. **TibebeJS/telegram-desktop-UI-clone**: Vue + Tailwind 實現
5. **vue-advanced-chat**: 完整聊天解決方案
6. **VantChatUI**: Vue 3 聊天元件庫

### 設計資源
7. **Telegram Mini Apps UI Kit (Figma)**: https://www.figma.com/community/file/1348989725141777736
8. **Flowbite Chat Bubble**: https://flowbite.com/docs/components/chat-bubble/

---

## ✅ 下一步行動

此分析文件已完成。要開始實作，請執行：

```bash
/start-work
```

這將啟動 Sisyphus 開始執行計畫，包含：
1. 安裝 TeleVue 函式庫
2. 設置 Telegram 顏色系統
3. 實作所有元件
4. 重構 GameView

---

*文件位置: `.sisyphus/drafts/telegram-ui-analysis.md`*  
*分析狀態: ✅ 完成*