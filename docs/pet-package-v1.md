# QPets 运行时角色包 v1

## 设计来源与兼容边界

本格式中的 `pet.json`、`spriteVersionNumber: 2` 和 `spritesheet.webp` 等基础约定参考了 [YaKun9/codex-pets](https://github.com/YaKun9/codex-pets)。Qpets 为独立桌面运行时增加了 `schemaVersion`、`renderType`、缩略图、对话资源、静态角色类型以及严格的文件与安全校验，所以上游 Codex 宠物目录不能视为可直接导入的 Qpets 角色包；导入前应按本文档重新封装并运行验证脚本。

该参考不包含上游现成宠物素材。上游宠物素材不适用其仓库根目录 MIT License，使用时必须同时遵守上游的 [素材授权总则](https://github.com/YaKun9/codex-pets/blob/main/ASSETS_LICENSE.md) 和对应宠物目录中的 `LICENSE.md`。

每个运行时角色包的根目录只能包含固定文件集合，禁止子目录、符号链接、源图、联系表、预览 GIF、验证报告和照片：

- `sprite-atlas-v2`：`pet.json`、`dialogues.json`、`thumbnail.png`、`spritesheet.webp`
- `static-image-v1`：`pet.json`、`dialogues.json`、`thumbnail.png`、`character.png`

路径字段不是可配置的别名：它们必须精确指向以上文件名，且不得复用同一个文件。

## `pet.json`

所有包必须有 `schemaVersion: 1`、稳定的 `id`、`displayName`、`renderType`、`thumbnailPath: "thumbnail.png"` 与 `dialoguesPath: "dialogues.json"`。

ID 只能使用小写 ASCII 字母、数字、`.`、`_`、`-`，长度 3–64；内置包必须使用 `qpet-` 前缀，用户包必须使用 `user.` 前缀。用户包不得覆盖内置 ID 或既有用户 ID。资源路径均相对包根目录，且不得包含 `..`、反斜杠、目录分隔符或绝对路径。

### `sprite-atlas-v2`

该类型必须声明：

```json
{
  "renderType": "sprite-atlas-v2",
  "spriteVersionNumber": 2,
  "spritesheetPath": "spritesheet.webp"
}
```

图集必须实际解码为带 alpha 的 RGBA WebP，固定为 `1536×2288` 像素，即 8 列 × 11 行，每格 `192×208`。行 0–8 分别为 `idle`、`running-right`、`running-left`、`waving`、`jumping`、`failed`、`waiting`、`running`、`review`；行 9–10 按顺时针每 22.5° 放置 16 个 look 方向。有效帧数为 `7,8,8,4,5,8,6,6,6,8,8`；每个有效格至少有 256 个非透明像素，其他格必须全透明。所有完全透明像素的 RGB 都必须为 `0,0,0`。

### `static-image-v1`

该类型必须声明 `characterPath: "character.png"`。它只提供静态形象；运行时不得把它当作 v2 帧图集播放。`character.png` 与 `thumbnail.png` 均须实际解码为带 alpha、且包含透明背景像素的 PNG。

## `dialogues.json`

最小格式为：

```json
{
  "schemaVersion": 1,
  "lines": { "idle": ["..."], "working": ["..."], "success": ["..."], "error": ["..."] }
}
```

`idle`、`working`、`success`、`error` 四个 key 都是必需项，且每个值必须是非空字符串数组。应用可为未知事件回退到 `idle`。

## 验证

使用项目 bundled Python 执行：

```powershell
& $PYTHON scripts/validate-pets.py --root src-tauri/resources/pets
```

脚本将 JSON 写到标准输出，非零退出表示至少一个包不合规。它不把旧的 `atlas-layout.json` 作为运行时引用来源；`pet.json` 是唯一的主资源声明。
