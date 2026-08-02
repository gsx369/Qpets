# Qpets

Qpets 是一个 Windows 优先的轻量桌面宠物。赵大美女是默认主角，赵二美女与俺滴老妹是内置可替换角色；用户也可以添加透明图片角色或导入完整 `.qpet` 动作包。

<p>
  <img src="src-tauri/resources/pets/qpet-z1-sunny-brim/thumbnail.png" width="150" alt="赵大美女">
  <img src="src-tauri/resources/pets/qpet-z2-violet-whisper/thumbnail.png" width="150" alt="赵二美女">
  <img src="src-tauri/resources/pets/qpet-gu-candied-haw/thumbnail.png" width="150" alt="俺滴老妹">
</p>

## 当前功能

- 透明无边框、始终置顶的桌宠窗口，以及独立设置窗口和系统托盘。
- 三个内置角色均提供 v2 8×11 动作图集；自定义角色支持完整动作包或静态透明图片。
- 单击互动、拖动反馈、滚轮缩放、闲置表演、对话气泡和 16 方向鼠标注视。
- 三个内置角色可即时切换，并可在本机修改显示名称和介绍；原始角色模型保持只读。
- PNG/WebP 静态角色导入，以及 `.qpet`/ZIP 完整角色包导入。
- 自定义角色的事务式安装和删除；删除当前角色时自动回退到默认角色。
- 所有角色都支持本地资料覆盖与一键恢复默认，不修改角色包内容。
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

## 参考与致谢

Qpets 角色图集的制作流程，以及 `pet.json`、`spriteVersionNumber: 2`、`spritesheet.webp` 等基础约定，参考了 [YaKun9/codex-pets](https://github.com/YaKun9/codex-pets)。Qpets 在此基础上定义了面向独立桌面程序的运行时角色包、对话资源、安全导入和本地角色管理规则，因此不承诺与上游角色目录直接互换；具体差异见 [角色包文档](docs/pet-package-v1.md)。

本仓库没有复制或分发 `codex-pets` 的网站代码和现成宠物素材。上游仓库级代码与文档采用 MIT License，宠物素材则遵循上游的素材总则和各角色目录自己的 `LICENSE.md`。完整说明见 [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md)。

## 隐私与许可

公开仓库不包含任何真人原始照片、生成过程目录或 QA 中间文件。代码使用 [MIT License](LICENSE)；角色美术不随 MIT 授权，参见 [ASSETS_LICENSE.md](ASSETS_LICENSE.md)。
