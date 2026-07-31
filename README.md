# Qpets

Qpets 是一个 Windows 优先的轻量桌面宠物。晴檐是默认主角，堇语与糖葫芦是内置可替换角色；用户也可以添加透明图片角色或导入完整 `.qpet` 动作包。

<p>
  <img src="src-tauri/resources/pets/qpet-z1-sunny-brim/thumbnail.png" width="150" alt="晴檐">
  <img src="src-tauri/resources/pets/qpet-z2-violet-whisper/thumbnail.png" width="150" alt="堇语">
  <img src="src-tauri/resources/pets/qpet-gu-candied-haw/thumbnail.png" width="150" alt="糖葫芦">
</p>

## 当前功能

- 透明无边框、始终置顶的桌宠窗口，以及独立设置窗口和系统托盘。
- v2 8×11 动作图集与静态透明图片两种渲染器。
- 单击互动、拖动反馈、滚轮缩放、闲置表演、对话气泡和 16 方向鼠标注视。
- 三个只读内置角色，可即时切换。
- PNG/WebP 静态角色导入，以及 `.qpet`/ZIP 完整角色包导入。
- 自定义角色的事务式安装和删除；删除当前角色时自动回退到晴檐。
- 设置和角色库由 Rust 后端统一持久化，两个窗口只订阅状态。

## 技术栈

- Tauri 2 / Rust
- Vue 3 / TypeScript / Vite
- Canvas 2D
- Vitest
- GitHub Actions / NSIS

## 本地开发

需要 Node.js、pnpm、Rust stable，以及带 Windows SDK 的 Visual Studio C++ Build Tools。

```powershell
pnpm install
pnpm check
pnpm tauri dev
```

只预览设置页：

```powershell
pnpm dev
```

浏览器访问 `http://localhost:1420/`；追加 `?view=pet` 可预览桌宠窗口的浏览器降级形态。

验证内置角色资源：

```powershell
python scripts/validate-pets.py --root src-tauri/resources/pets
```

## 角色包

完整格式见 [docs/pet-package-v1.md](docs/pet-package-v1.md)。角色包是声明式资源，不允许携带或执行脚本。内置资源放在 `src-tauri/resources/pets`；用户资源安装到系统应用数据目录，不回写程序目录。

## 隐私与许可

公开仓库不包含任何真人原始照片、生成过程目录或 QA 中间文件。代码使用 [MIT License](LICENSE)；角色美术不随 MIT 授权，参见 [ASSETS_LICENSE.md](ASSETS_LICENSE.md)。
