# 🗼 Project Babel (巴別塔計畫)

> **GGJ 2026 Submission - Tainan Team B**
> 
> *"In a world of surveillance, four strangers use a forgotten language to speak the truth."*

![Banner Image](docs/assets/banner.png)

## 📖 專案簡介 (About)
**Project Babel** 是一款多人合作的社會反烏托邦解謎遊戲。
四名玩家身處不同的極權國家，面對各自的網路審查防火牆。為了傳遞真相，你們必須使用一套由 26 個原始符號組成的「未定義語言」，在充滿 `****` 與雜訊的聊天室中，拼湊出自由的拼圖。

詳細設計文件請參閱：[Game Design Document](docs/docs.md)

## 🎮 核心機制 (Mechanics)
- **語言解謎 (Undefined Language):** 使用 26 個原創表意符號溝通。
- **審查過濾 (The Mask):** 敏感詞會被各國防火牆即時遮蔽。
- **語音輸入 (Voice Input):** 支援 Web Speech API，用「說」的輸入符號。
- **跨平台 (Cross-Platform):** 響應式網頁設計，支援 PC 與 Mobile。

---

## 🛠️ 技術堆疊 (Tech Stack)

我們採用 **Rust** + **Web** 技術，達成極致輕量化 (<10MB) 與高併發效能。

| Component | Tech | Why? |
| :--- | :--- | :--- |
| **Backend** | **Rust (Axum)** | 高效能、記憶體安全、單一執行檔部署。 |
| **Realtime** | **Socketioxide** | Rust 實作的 Socket.io，處理即時通訊。 |
| **Frontend** | **Vue 3 + Vite** | 快速開發元件化 UI。 |
| **Styling** | **Tailwind CSS** | 快速實作多種聊天軟體 (Telegram/Line) 主題切換。 |
| **Voice** | **Web Speech API** | 瀏覽器原生支援，無須外部依賴。 |

---

## 🚀 快速開始 (Quick Start)

### 前置需求 (Prerequisites)
- [Node.js](https://nodejs.org/) (v18+)
- [Rust](https://www.rust-lang.org/) (Latest Stable)

### 安裝與執行 (Development)

1. **Clone Repo**
   ```bash
    git clone https://github.com/YourUser/project-babel.git
    cd project-babel
    ```

2. **安裝依賴**
   ```bash
    cd frontend
    bun install
   ```

3. **啟動開發伺服器**
   ```bash
    bun run dev
   ```

   Frontend: `http://localhost:5173`

   *Note: Backend will be added in a future update.*



---

## 🏆 成就列表 (Diversifiers Checklist)

我們在本次 Game Jam 挑戰了以下成就：

| Status | 類別 | 成就名稱 | 實作說明 |
| --- | --- | --- | --- |
| ✅ | **Narrative** | **Undefined Language** | 核心玩法：全符號與人造語音溝通。 |
| ✅ | **Code** | **Cartridge Ready** | 專案編譯後體積極小，不依賴大型引擎。 |
| ✅ | **Sponsored** | **Cross-Platform Play** | RWD 網頁架構，手機電腦皆可玩。 |
| 🚧 | **Accessibility** | **Hands Off** | 支援全語音輸入控制。 |
| 🚧 | **Accessibility** | **Out of Sight** | 支援 TTS 訊息朗讀與音效回饋。 |
| ⏳ | **Code** | **Random encounter** | 隨機生成的國家規則與禁字表。 |
| ⏳ | **Narrative** | **Stay Local** | 加入在地文化梗的禁字庫。 |

---

## 📂 目錄結構 (Structure)

* `/backend`: Rust Axum 伺服器源碼。
* `/frontend`: Vue 3 前端源碼。
* `/shared`: 前後端共用的設定檔 (如 `protocol.json`)。
* `/docs`: 遊戲設計文件與素材。

## 📜 License

MIT License
