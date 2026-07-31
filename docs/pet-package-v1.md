# QPets 运行时角色包 v1

每个内置或导入后的运行时角色包只包含 `pet.json`、`dialogues.json`、`thumbnail.png`，以及一种主资源：v2 动画包为 `spritesheet.webp`，静态包为 `character.png`。制作源图、联系表、预览 GIF、验证报告和照片不进入运行时包。

## `pet.json`

所有包必须有 `schemaVersion: 1`、稳定的 `id`、`displayName`、`renderType`、`thumbnailPath: "thumbnail.png"` 与 `dialoguesPath: "dialogues.json"`。资源路径均相对包根目录，且不得包含 `..`。

### `sprite-atlas-v2`

该类型必须声明：

```json
{
  "renderType": "sprite-atlas-v2",
  "spriteVersionNumber": 2,
  "spritesheetPath": "spritesheet.webp"
}
```

图集固定为透明 RGBA WebP，`1536×2288` 像素，即 8 列 × 11 行，每格 `192×208`。行 0–8 分别为 `idle`、`running-right`、`running-left`、`waving`、`jumping`、`failed`、`waiting`、`running`、`review`；行 9–10 按顺时针每 22.5° 放置 16 个 look 方向。有效帧数为 `7,8,8,4,5,8,6,6,6,8,8`；其他格必须全透明。

### `static-image-v1`

该类型必须声明 `characterPath: "character.png"`。它只提供静态形象；运行时不得把它当作 v2 帧图集播放。`character.png` 与 `thumbnail.png` 均须为带 alpha 的 PNG。

## `dialogues.json`

最小格式为：

```json
{
  "schemaVersion": 1,
  "lines": { "idle": ["..."], "working": ["..."], "success": ["..."], "error": ["..."] }
}
```

每个 key 的值必须是非空字符串数组。应用可为未知事件回退到 `idle`。

## 验证

使用项目 bundled Python 执行：

```powershell
& $PYTHON scripts/validate-pets.py --root src-tauri/resources/pets
```

脚本将 JSON 写到标准输出，非零退出表示至少一个包不合规。它不把旧的 `atlas-layout.json` 作为运行时引用来源；`pet.json` 是唯一的主资源声明。
