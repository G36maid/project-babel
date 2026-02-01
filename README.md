# 🗼 Project Babel (巴別塔計畫)

> **GGJ 2026 Submission - Tainan Team B**
> 
> *"In a world of surveillance, four strangers use a forgotten language to speak the truth."*

![Banner Image](docs/assets/banner.png)

## 📖 專案簡介 (About)
**Project Babel** 是一款多人合作的社會反烏托邦解謎遊戲。
四名玩家身處不同的極權國家，面對各自的網路審查防火牆。為了傳遞真相，你們必須使用一套由 26 個原始符號組成的「未定義語言」，在充滿 `****` 與雜訊的聊天室中，拼湊出自由的拼圖。

詳細設計文件請參閱：[Game Design Document](docs/game_design.md)
開發紀錄請參閱：[Process Journal](docs/process_journal.md)

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

### ⭐ 推薦方式：Docker Compose (最快、最穩定)
這是開發者最推薦的執行方式，能確保後端 Rust 環境與前端 Vue 環境完全一致。

```bash
# 1. Clone Repo
git clone https://github.com/G36maid/project-babel.git
cd project-babel

# 2. 啟動所有服務
docker-compose up -d
```

**服務位置**:
- 前端介面: `http://localhost:8080`
- 後端 API: `http://localhost:3000`
- API 文檔: `http://localhost:3000/swagger-ui/`

**常用管理指令**:
```bash
docker-compose logs -f        # 查看即時日誌
docker-compose down           # 停止並移除容器
docker-compose up -d --build  # 更新程式碼後重新建置
```

---

### 其他方式 (開發調試)

#### 方法 2: Local Development (本機開發)
適合需要進行程式碼熱重載 (Hot Reload) 的開發者。

1. **安裝環境**:
   - [Bun](https://bun.sh/) (前端推薦使用，速度快 4-6 倍)
   - [Rust](https://www.rust-lang.org/) (後端)

2. **執行**:
   - Terminal 1 (後端): `cd backend && cargo run`
   - Terminal 2 (前端): `cd frontend && bun run dev`

#### 方法 3: Hybrid Mode (混合模式)
如果你只想修改前端，可以讓後端跑在 Docker：
```bash
docker-compose up -d backend
cd frontend && bun run dev
```



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
