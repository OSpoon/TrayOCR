# starter-tauri-app

用于桌面应用开发的模板项目，核心栈：

- Vue 3 + TypeScript + Vite 6
- Tauri 2（Rust）
- Tailwind CSS 4 + shadcn-vue
- ESLint + simple-git-hooks + lint-staged

---

## 获取模板

### GitHub「Use this template」

1. 在 GitHub 打开该模板仓库。
2. 点击 **Use this template** -> **Create a new repository**。
3. 填写你的新仓库名（例如 `my-desktop-app`），创建并克隆到本地。
4. 进入目录，先完成“项目初始化”（见下方表格）。
5. 完成改名后，再安装依赖并启动项目。

```bash
pnpm install
pnpm tauri dev
```

---

## 项目初始化（表格对照）

示例目标值：

- 项目名：`my-desktop-app`
- 产品名：`My Desktop App`
- 包标识：`com.acme.mydesktop`
- Rust lib 名：`my_desktop_app_lib`

| 序号 | 文件 | 修改项 | 原值 | 新值 |
| --- | --- | --- | --- | --- |
| 1 | `package.json` | `name` | `starter-tauri-app` | `my-desktop-app` |
| 2 | `src-tauri/Cargo.toml` | `[package].name` | `starter-tauri-app` | `my-desktop-app` |
| 3 | `src-tauri/Cargo.toml` | `[lib].name` | `starter_tauri_app_lib` | `my_desktop_app_lib` |
| 4 | `src-tauri/src/main.rs` | `run()` 调用 | `starter_tauri_app_lib::run()` | `my_desktop_app_lib::run()` |
| 5 | `src-tauri/tauri.conf.json` | `productName` | `starter-tauri-app` | `My Desktop App` |
| 6 | `src-tauri/tauri.conf.json` | `identifier` | `com.osp.starter-tauri-app` | `com.acme.mydesktop` |
| 7 | `src-tauri/tauri.conf.json` | `app.windows[0].title` | `starter-tauri-app` | `My Desktop App` |

---

## 改名后自检清单

- `pnpm tauri dev` 可以正常启动
- 应用窗口标题已变为新名称
- 控制台无 `*_lib::run()` 相关命名错误
- `identifier` 已改为你的正式命名（建议反向域名）

---

## 使用 GitHub Releases 更新应用（Updater 配置流程）

本模板已集成 `@tauri-apps/plugin-updater` / `tauri-plugin-updater`。要让“检测更新”真正工作，需要把 **签名密钥 + GitHub Release + latest.json** 串起来（Tauri Updater 强制要求签名，不能关闭）。

参考官方文档：[更新 | Tauri](https://tauri.app/zh-cn/plugin/updater/)

### 1) 生成 updater 签名密钥（本地执行一次）

在项目根目录运行（会生成私钥文件与对应公钥）：

```bash
pnpm tauri signer generate -w ./.tauri/updater.key
```

生成后你会得到：

- 私钥：`.tauri/updater.key`（**不要提交到 git**）
- 公钥：`.tauri/updater.key.pub`（可公开，用于写入配置）

### 2) 把公钥写入 `src-tauri/tauri.conf.json`

将 `plugins.updater.pubkey` 替换为公钥文件内容（注意：是“内容”，不是路径）。

同时确保 endpoint 指向你们仓库的 `latest.json`：

- `https://github.com/<owner>/<repo>/releases/latest/download/latest.json`

### 3) 打开 updater 产物生成

确认 `src-tauri/tauri.conf.json` 中已启用：

- `bundle.createUpdaterArtifacts: true`

它会在打包时生成 updater 所需的签名产物，并由 Tauri Action 生成 `latest.json` 供 GitHub Releases 分发。

### 4) 配置 GitHub Actions Secrets（用于签名）

在仓库 `Settings -> Secrets and variables -> Actions` 添加：

- `TAURI_SIGNING_PRIVATE_KEY`
  - 填：私钥文件内容（或私钥路径，按你们习惯；建议内容更稳）
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`
  - 若生成私钥时设置了密码则填写，否则可留空/不建

并确认仓库 Actions 允许写入 release：

- `Settings -> Actions -> General -> Workflow permissions` 选择 **Read and write permissions**

### 5) 发布流程（触发打包并生成 GitHub Release）

执行：

```bash
npm run release
```

它会完成：更新版本号（含 `tauri.conf.json` / `Cargo.toml`）→ 提交 → 打 tag（`app-vX.Y.Z`）→ push → 触发工作流打包 → 生成 Release（草稿）及 `latest.json`。

### 6) 验证更新

1. 先安装旧版本（或保留旧版本应用）。
2. 用 `npm run release` 发一个更高版本，等待 GitHub Release 产物完整。
3. 打开旧版本应用，点击页面上的 `Check for updates`：
   - 能拉到 `latest.json` 并提示更新
   - 能下载并安装（Windows 安装更新时可能自动退出属于正常行为）