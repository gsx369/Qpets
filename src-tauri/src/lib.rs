//! Trusted local backend for Qpets.
//!
//! Both webviews are read-only clients. This module is the only writer for the
//! settings file and user pet library, which keeps multi-window updates ordered.

use image::{DynamicImage, GenericImageView, ImageFormat, ImageReader, Limits, RgbaImage};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    fs,
    fs::{File, OpenOptions},
    io::{self, BufReader, Write},
    path::{Component, Path, PathBuf},
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};
use tauri::{
    menu::{Menu, MenuItem},
    tray::TrayIconBuilder,
    AppHandle, Emitter, LogicalSize, Manager, State, WebviewWindow,
};
use tauri_plugin_autostart::{MacosLauncher, ManagerExt};
use zip::ZipArchive;

const STATE_FILE: &str = "state.json";
const PETS_DIR: &str = "pets";
const PACKAGES_DIR: &str = "packages";
const STAGING_DIR: &str = ".staging";
const TRASH_DIR: &str = ".trash";
const DEFAULT_PET_ID: &str = "qpet-z1-sunny-brim";
const MAX_ARCHIVE_FILES: usize = 24;
const MAX_ARCHIVE_SOURCE_BYTES: u64 = 48 * 1024 * 1024;
const MAX_ARCHIVE_FILE_BYTES: u64 = 32 * 1024 * 1024;
const MAX_ARCHIVE_BYTES: u64 = 96 * 1024 * 1024;
const MAX_IMAGE_ALLOC_BYTES: u64 = 96 * 1024 * 1024;
const MIN_FRAME_VISIBLE_PIXELS: usize = 256;
const PET_BASE_WIDTH: f64 = 256.0;
const PET_BASE_HEIGHT: f64 = 256.0;

type CommandResult<T> = Result<T, String>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub always_on_top: bool,
    pub click_through: bool,
    pub follow_cursor: bool,
    pub show_bubble: bool,
    pub start_with_windows: bool,
    pub volume: f32,
    pub scale: f64,
    pub idle_interval_seconds: u32,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            always_on_top: true,
            click_through: false,
            follow_cursor: true,
            show_bubble: true,
            start_with_windows: false,
            volume: 0.55,
            scale: 1.0,
            idle_interval_seconds: 30,
        }
    }
}

impl Settings {
    fn validate(&self) -> CommandResult<()> {
        if !(0.0..=1.0).contains(&self.volume) {
            return Err("音量必须位于 0 到 1 之间".into());
        }
        if !(0.6..=1.6).contains(&self.scale) {
            return Err("桌宠缩放必须位于 60% 到 160% 之间".into());
        }
        if !(20..=60).contains(&self.idle_interval_seconds) {
            return Err("闲置表演间隔必须位于 20 到 60 秒之间".into());
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "kebab-case")]
pub enum RenderType {
    SpriteAtlasV2,
    StaticImageV1,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum PetSource {
    Builtin,
    User,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PetDescriptor {
    pub id: String,
    pub display_name: String,
    pub description: String,
    pub render_type: RenderType,
    pub source: PetSource,
    pub deletable: bool,
    pub asset_path: String,
    pub thumbnail_path: String,
    pub dialogues: HashMap<String, Vec<String>>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AppState {
    pub settings: Settings,
    pub selected_pet_id: String,
    pub pets: Vec<PetDescriptor>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PersistedState {
    library_schema_version: u32,
    #[serde(default)]
    settings: Settings,
    #[serde(default = "default_pet_id")]
    selected_pet_id: String,
    #[serde(default)]
    user_pets: Vec<UserPetRecord>,
}

impl Default for PersistedState {
    fn default() -> Self {
        Self {
            library_schema_version: 1,
            settings: Settings::default(),
            selected_pet_id: default_pet_id(),
            user_pets: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct UserPetRecord {
    id: String,
    installed_at: u64,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PetIndex {
    schema_version: u32,
    default_pet_id: String,
    pets: Vec<PetIndexEntry>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PetIndexEntry {
    id: String,
    package_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct PetManifest {
    schema_version: u32,
    id: String,
    display_name: String,
    #[serde(default)]
    description: String,
    render_type: RenderType,
    #[serde(skip_serializing_if = "Option::is_none")]
    sprite_version_number: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    spritesheet_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    character_path: Option<String>,
    thumbnail_path: String,
    dialogues_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct DialogueFile {
    schema_version: u32,
    lines: HashMap<String, Vec<String>>,
}

struct Store {
    root: PathBuf,
    builtins: Vec<PetDescriptor>,
    persisted: PersistedState,
}

impl Store {
    fn open(root: PathBuf, resource_pets: PathBuf) -> CommandResult<Self> {
        for directory in [
            root.join(PETS_DIR).join(PACKAGES_DIR),
            root.join(PETS_DIR).join(STAGING_DIR),
            root.join(PETS_DIR).join(TRASH_DIR),
        ] {
            fs::create_dir_all(directory).map_err(error_text)?;
        }

        let builtins = load_builtins(&resource_pets)?;
        let state_path = root.join(STATE_FILE);
        let mut persisted = if state_path.exists() {
            read_json(&state_path)?
        } else if state_path.with_extension("json.bak").exists() {
            read_json(&state_path.with_extension("json.bak"))?
        } else {
            PersistedState::default()
        };

        if persisted.library_schema_version != 1 {
            return Err("不支持的本地角色库版本".into());
        }
        persisted.settings.validate()?;

        let packages = root.join(PETS_DIR).join(PACKAGES_DIR);
        persisted.user_pets.retain(|record| {
            if !is_valid_pet_id(&record.id, false) {
                return false;
            }
            descriptor_from_package(&packages.join(&record.id), PetSource::User, false, false)
                .is_ok_and(|descriptor| descriptor.id == record.id)
        });

        let selected_exists = builtins
            .iter()
            .any(|pet| pet.id == persisted.selected_pet_id)
            || persisted
                .user_pets
                .iter()
                .any(|pet| pet.id == persisted.selected_pet_id);
        if !selected_exists {
            persisted.selected_pet_id = DEFAULT_PET_ID.into();
        }

        let store = Self {
            root,
            builtins,
            persisted,
        };
        store.save()?;
        Ok(store)
    }

    fn packages_dir(&self) -> PathBuf {
        self.root.join(PETS_DIR).join(PACKAGES_DIR)
    }

    fn staging_dir(&self) -> PathBuf {
        self.root.join(PETS_DIR).join(STAGING_DIR)
    }

    fn trash_dir(&self) -> PathBuf {
        self.root.join(PETS_DIR).join(TRASH_DIR)
    }

    fn package_dir(&self, id: &str) -> CommandResult<PathBuf> {
        if !is_valid_pet_id(id, false) {
            return Err("自定义角色 ID 无效".into());
        }
        Ok(self.packages_dir().join(id))
    }

    fn save(&self) -> CommandResult<()> {
        write_json_transactional(&self.root.join(STATE_FILE), &self.persisted)
    }

    fn has_pet(&self, id: &str) -> bool {
        self.builtins.iter().any(|pet| pet.id == id)
            || self.persisted.user_pets.iter().any(|pet| pet.id == id)
    }

    fn app_state(&self) -> AppState {
        let mut pets = self.builtins.clone();
        pets.extend(self.persisted.user_pets.iter().filter_map(|record| {
            self.package_dir(&record.id).ok().and_then(|directory| {
                descriptor_from_package(&directory, PetSource::User, false, false)
                    .ok()
                    .filter(|descriptor| descriptor.id == record.id)
            })
        }));
        AppState {
            settings: self.persisted.settings.clone(),
            selected_pet_id: self.persisted.selected_pet_id.clone(),
            pets,
        }
    }
}

fn default_pet_id() -> String {
    DEFAULT_PET_ID.into()
}

fn error_text(error: impl std::fmt::Display) -> String {
    error.to_string()
}

fn operation_id() -> String {
    let micros = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_micros();
    format!("{micros}-{}", std::process::id())
}

fn unix_seconds() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn read_json<T: DeserializeOwned>(path: &Path) -> CommandResult<T> {
    let bytes = fs::read(path).map_err(error_text)?;
    serde_json::from_slice(&bytes).map_err(error_text)
}

fn write_json_file<T: Serialize>(path: &Path, value: &T) -> CommandResult<()> {
    let bytes = serde_json::to_vec_pretty(value).map_err(error_text)?;
    let mut file = File::create(path).map_err(error_text)?;
    file.write_all(&bytes).map_err(error_text)?;
    file.sync_all().map_err(error_text)
}

/// Uses a recoverable temp -> backup -> final transaction. On startup the
/// backup is accepted if a crash happened between the two renames.
fn write_json_transactional<T: Serialize>(path: &Path, value: &T) -> CommandResult<()> {
    let parent = path
        .parent()
        .ok_or_else(|| "状态文件路径无效".to_string())?;
    fs::create_dir_all(parent).map_err(error_text)?;
    let temporary = parent.join(format!(".state-{}.tmp", operation_id()));
    let backup = path.with_extension("json.bak");
    write_json_file(&temporary, value)?;

    if backup.exists() {
        fs::remove_file(&backup).map_err(error_text)?;
    }
    if path.exists() {
        fs::rename(path, &backup).map_err(error_text)?;
    }
    if let Err(error) = fs::rename(&temporary, path) {
        if backup.exists() {
            let _ = fs::rename(&backup, path);
        }
        let _ = fs::remove_file(&temporary);
        return Err(error_text(error));
    }
    if backup.exists() {
        let _ = fs::remove_file(backup);
    }
    Ok(())
}

fn load_builtins(root: &Path) -> CommandResult<Vec<PetDescriptor>> {
    let index: PetIndex = read_json(&root.join("index.json"))?;
    if index.schema_version != 1 || index.default_pet_id != DEFAULT_PET_ID {
        return Err("内置角色索引版本或默认角色不正确".into());
    }

    let mut pets = Vec::with_capacity(index.pets.len());
    for entry in index.pets {
        if !is_safe_file_name(&entry.package_path)
            || !is_valid_pet_id(&entry.id, true)
            || entry.id != entry.package_path
        {
            return Err(format!("内置角色 {} 的包路径不一致", entry.id));
        }
        let descriptor = descriptor_from_package(
            &root.join(&entry.package_path),
            PetSource::Builtin,
            true,
            false,
        )?;
        if descriptor.id != entry.id {
            return Err(format!("内置角色 {} 的清单 ID 不一致", entry.id));
        }
        pets.push(descriptor);
    }
    if !pets.iter().any(|pet| pet.id == DEFAULT_PET_ID) {
        return Err("缺少默认角色晴檐".into());
    }
    Ok(pets)
}

fn is_safe_file_name(value: &str) -> bool {
    let path = Path::new(value);
    !value.is_empty()
        && path.components().count() == 1
        && matches!(path.components().next(), Some(Component::Normal(_)))
}

fn is_valid_pet_id(value: &str, allow_builtin_namespace: bool) -> bool {
    let has_expected_namespace = if allow_builtin_namespace {
        value.starts_with("qpet-")
    } else {
        value.starts_with("user.")
    };
    if !(6..=64).contains(&value.len()) || !has_expected_namespace {
        return false;
    }
    let Some(last) = value.chars().last() else {
        return false;
    };
    last.is_ascii_alphanumeric()
        && value.chars().all(|character| {
            character.is_ascii_lowercase()
                || character.is_ascii_digit()
                || matches!(character, '.' | '_' | '-')
        })
}

fn validate_manifest(manifest: &PetManifest, source: PetSource) -> CommandResult<()> {
    if manifest.schema_version != 1 {
        return Err("角色包 schemaVersion 必须为 1".into());
    }
    if !is_valid_pet_id(&manifest.id, source == PetSource::Builtin) {
        return Err("角色 ID 格式无效或使用了保留命名空间".into());
    }
    if manifest.display_name.trim().is_empty() || manifest.display_name.chars().count() > 64 {
        return Err("角色名称不能为空且不能超过 64 个字符".into());
    }
    if manifest.description.chars().count() > 280 {
        return Err("角色描述不能超过 280 个字符".into());
    }
    if manifest.thumbnail_path != "thumbnail.png" || manifest.dialogues_path != "dialogues.json" {
        return Err("v1 角色包必须使用固定的 thumbnail.png 和 dialogues.json".into());
    }
    match manifest.render_type {
        RenderType::SpriteAtlasV2 => {
            if manifest.sprite_version_number != Some(2)
                || manifest.spritesheet_path.as_deref() != Some("spritesheet.webp")
                || manifest.character_path.is_some()
            {
                return Err("动态角色必须只声明 spriteVersionNumber 2 和 spritesheet.webp".into());
            }
        }
        RenderType::StaticImageV1 => {
            if manifest.character_path.as_deref() != Some("character.png")
                || manifest.sprite_version_number.is_some()
                || manifest.spritesheet_path.is_some()
            {
                return Err("静态角色必须只声明 character.png".into());
            }
        }
    }
    Ok(())
}

fn validate_package_files(directory: &Path, render_type: RenderType) -> CommandResult<()> {
    let expected_names = match render_type {
        RenderType::SpriteAtlasV2 => [
            "pet.json",
            "dialogues.json",
            "thumbnail.png",
            "spritesheet.webp",
        ],
        RenderType::StaticImageV1 => [
            "pet.json",
            "dialogues.json",
            "thumbnail.png",
            "character.png",
        ],
    };
    let expected: HashSet<String> = expected_names.into_iter().map(str::to_owned).collect();
    let mut actual = HashSet::new();
    for entry in fs::read_dir(directory).map_err(error_text)? {
        let entry = entry.map_err(error_text)?;
        if !entry.file_type().map_err(error_text)?.is_file() {
            return Err("角色包只能包含规定的根目录文件".into());
        }
        let name = entry
            .file_name()
            .into_string()
            .map_err(|_| "角色包包含无效文件名".to_string())?;
        actual.insert(name);
    }
    if actual != expected {
        return Err("角色包文件集合与 renderType 不匹配".into());
    }
    Ok(())
}

fn decode_image(
    path: &Path,
    expected_format: ImageFormat,
    max_width: u32,
    max_height: u32,
) -> CommandResult<DynamicImage> {
    let mut reader = ImageReader::open(path)
        .map_err(error_text)?
        .with_guessed_format()
        .map_err(error_text)?;
    if reader.format() != Some(expected_format) {
        return Err(format!("{} 的实际图片格式与文件名不一致", path.display()));
    }
    let mut limits = Limits::default();
    limits.max_image_width = Some(max_width);
    limits.max_image_height = Some(max_height);
    limits.max_alloc = Some(MAX_IMAGE_ALLOC_BYTES);
    reader.limits(limits);
    reader.decode().map_err(error_text)
}

fn require_transparent_image(image: &DynamicImage) -> CommandResult<RgbaImage> {
    if !image.color().has_alpha() {
        return Err("角色图片必须包含透明通道".into());
    }
    let (width, height) = image.dimensions();
    if width == 0
        || height == 0
        || width > 4096
        || height > 4096
        || u64::from(width) * u64::from(height) > 16_777_216
    {
        return Err("角色图片尺寸无效或超过 4096×4096".into());
    }
    let rgba = image.to_rgba8();
    if rgba.pixels().filter(|pixel| pixel[3] > 0).count() < MIN_FRAME_VISIBLE_PIXELS {
        return Err("角色图片缺少足够的可见像素".into());
    }
    if !rgba.pixels().any(|pixel| pixel[3] < 255) {
        return Err("角色图片没有透明背景".into());
    }
    Ok(rgba)
}

fn validate_v2_atlas(image: &DynamicImage) -> CommandResult<RgbaImage> {
    if image.dimensions() != (1536, 2288) || !image.color().has_alpha() {
        return Err("v2 动作图集必须为带透明通道的 1536×2288 图片".into());
    }
    let rgba = image.to_rgba8();
    if rgba
        .pixels()
        .any(|pixel| pixel[3] == 0 && pixel[0..3] != [0, 0, 0])
    {
        return Err("v2 图集的透明像素必须清空 RGB 通道".into());
    }
    let used_columns = [7usize, 8, 8, 4, 5, 8, 6, 6, 6, 8, 8];
    for (row, used) in used_columns.into_iter().enumerate() {
        for column in 0..8usize {
            let visible_pixels = (0..208usize)
                .flat_map(|y| (0..192usize).map(move |x| (x, y)))
                .filter(|(x, y)| {
                    rgba.get_pixel((column * 192 + x) as u32, (row * 208 + y) as u32)[3] > 0
                })
                .count();
            if column < used && visible_pixels < MIN_FRAME_VISIBLE_PIXELS {
                return Err(format!("v2 图集第 {} 行第 {} 格缺少有效帧", row, column));
            }
            if column >= used && visible_pixels > 0 {
                return Err(format!(
                    "v2 图集第 {} 行第 {} 个保留格必须透明",
                    row, column
                ));
            }
        }
    }
    Ok(rgba)
}

fn validate_dialogues(path: &Path) -> CommandResult<HashMap<String, Vec<String>>> {
    let file: DialogueFile = read_json(path)?;
    if file.schema_version != 1 || file.lines.is_empty() {
        return Err("dialogues.json 版本无效或没有台词".into());
    }
    if ["idle", "working", "success", "error"]
        .into_iter()
        .any(|key| !file.lines.contains_key(key))
    {
        return Err("dialogues.json 必须包含 idle、working、success、error".into());
    }
    if file.lines.values().any(|lines| {
        lines.is_empty()
            || lines
                .iter()
                .any(|line| line.trim().is_empty() || line.chars().count() > 120)
    }) {
        return Err("dialogues.json 包含空台词或过长台词".into());
    }
    Ok(file.lines)
}

fn main_asset_path<'a>(manifest: &'a PetManifest) -> CommandResult<&'a str> {
    match manifest.render_type {
        RenderType::SpriteAtlasV2 => manifest
            .spritesheet_path
            .as_deref()
            .ok_or_else(|| "动态角色缺少 spritesheetPath".into()),
        RenderType::StaticImageV1 => manifest
            .character_path
            .as_deref()
            .ok_or_else(|| "静态角色缺少 characterPath".into()),
    }
}

fn generate_thumbnail(
    image: &DynamicImage,
    render_type: RenderType,
    destination: &Path,
) -> CommandResult<()> {
    let source = match render_type {
        RenderType::SpriteAtlasV2 => image.crop_imm(6 * 192, 0, 192, 208),
        RenderType::StaticImageV1 => image.clone(),
    };
    source
        .thumbnail(256, 256)
        .save_with_format(destination, image::ImageFormat::Png)
        .map_err(error_text)
}

fn validate_thumbnail(path: &Path) -> CommandResult<()> {
    let image = decode_image(path, ImageFormat::Png, 512, 512)?;
    require_transparent_image(&image)?;
    Ok(())
}

fn descriptor_from_package(
    directory: &Path,
    source: PetSource,
    validate_pixels: bool,
    refresh_thumbnail: bool,
) -> CommandResult<PetDescriptor> {
    let manifest: PetManifest = read_json(&directory.join("pet.json"))?;
    validate_manifest(&manifest, source)?;
    validate_package_files(directory, manifest.render_type)?;
    let asset = directory.join(main_asset_path(&manifest)?);
    let thumbnail = directory.join(&manifest.thumbnail_path);
    let dialogues = validate_dialogues(&directory.join(&manifest.dialogues_path))?;

    if validate_pixels || refresh_thumbnail {
        let image = match manifest.render_type {
            RenderType::SpriteAtlasV2 => decode_image(&asset, ImageFormat::WebP, 1536, 2288)?,
            RenderType::StaticImageV1 => decode_image(&asset, ImageFormat::Png, 4096, 4096)?,
        };
        match manifest.render_type {
            RenderType::SpriteAtlasV2 => {
                validate_v2_atlas(&image)?;
            }
            RenderType::StaticImageV1 => {
                require_transparent_image(&image)?;
            }
        }
        if refresh_thumbnail {
            generate_thumbnail(&image, manifest.render_type, &thumbnail)?;
        }
    }
    if !asset.is_file() || !thumbnail.is_file() {
        return Err("角色包缺少主资源或缩略图".into());
    }
    validate_thumbnail(&thumbnail)?;

    Ok(PetDescriptor {
        id: manifest.id,
        display_name: manifest.display_name,
        description: manifest.description,
        render_type: manifest.render_type,
        source,
        deletable: source == PetSource::User,
        asset_path: asset.to_string_lossy().into_owned(),
        thumbnail_path: thumbnail.to_string_lossy().into_owned(),
        dialogues,
    })
}

fn slug(value: &str) -> String {
    value
        .chars()
        .filter_map(|character| {
            if character.is_ascii_alphanumeric() {
                Some(character.to_ascii_lowercase())
            } else if matches!(character, '-' | '_') {
                Some(character)
            } else if character.is_whitespace() {
                Some('-')
            } else {
                None
            }
        })
        .take(32)
        .collect::<String>()
        .trim_matches('-')
        .to_owned()
}

fn default_dialogues(name: &str) -> DialogueFile {
    DialogueFile {
        schema_version: 1,
        lines: HashMap::from([
            ("idle".into(), vec![format!("{name}在这里陪着你。")]),
            ("working".into(), vec!["我正在认真看看。".into()]),
            ("success".into(), vec!["好呀，完成啦。".into()]),
            ("error".into(), vec!["别着急，我们再试试。".into()]),
        ]),
    }
}

fn install_staged(store: &mut Store, staging: &Path) -> CommandResult<PetDescriptor> {
    let descriptor = descriptor_from_package(staging, PetSource::User, true, true)?;
    let verified = descriptor_from_package(staging, PetSource::User, true, false)?;
    if verified.id != descriptor.id {
        return Err("角色包在安装验证期间发生变化".into());
    }
    if store.has_pet(&descriptor.id) {
        return Err(format!("角色 ID {} 已存在", descriptor.id));
    }

    let destination = store.package_dir(&descriptor.id)?;
    fs::rename(staging, &destination).map_err(error_text)?;
    let previous_persisted = store.persisted.clone();
    store.persisted.user_pets.push(UserPetRecord {
        id: descriptor.id.clone(),
        installed_at: unix_seconds(),
    });
    store.persisted.selected_pet_id = descriptor.id.clone();

    if let Err(error) = store.save() {
        store.persisted = previous_persisted;
        return match fs::rename(&destination, staging) {
            Ok(()) => Err(error),
            Err(rollback_error) => Err(format!("{error}；角色目录回滚失败：{rollback_error}")),
        };
    }
    match descriptor_from_package(&destination, PetSource::User, false, false) {
        Ok(installed) => Ok(installed),
        Err(error) => {
            store.persisted = previous_persisted;
            let state_rollback = store.save();
            let directory_rollback = fs::rename(&destination, staging);
            match (state_rollback, directory_rollback) {
                (Ok(()), Ok(())) => Err(error),
                (state_result, directory_result) => Err(format!(
                    "{error}；安装回滚不完整（状态：{}，目录：{}）",
                    state_result.err().unwrap_or_else(|| "成功".into()),
                    directory_result
                        .err()
                        .map(|value| value.to_string())
                        .unwrap_or_else(|| "成功".into())
                )),
            }
        }
    }
}

fn extract_qpet(source: &Path, destination: &Path) -> CommandResult<()> {
    if fs::metadata(source).map_err(error_text)?.len() > MAX_ARCHIVE_SOURCE_BYTES {
        return Err("角色包压缩文件超过 48 MiB 限制".into());
    }
    let file = File::open(source).map_err(error_text)?;
    let mut archive = ZipArchive::new(BufReader::new(file)).map_err(error_text)?;
    if archive.len() == 0 || archive.len() > MAX_ARCHIVE_FILES {
        return Err("角色包文件数量无效".into());
    }
    let allowed = HashSet::from([
        "pet.json",
        "dialogues.json",
        "thumbnail.png",
        "spritesheet.webp",
        "character.png",
    ]);
    let mut seen = HashSet::new();
    let mut total = 0u64;

    fs::create_dir_all(destination).map_err(error_text)?;
    for index in 0..archive.len() {
        let mut entry = archive.by_index(index).map_err(error_text)?;
        if entry.is_dir() {
            continue;
        }
        if entry
            .unix_mode()
            .is_some_and(|mode| mode & 0o170000 == 0o120000)
        {
            return Err("角色包不能包含符号链接".into());
        }
        let enclosed = entry
            .enclosed_name()
            .ok_or_else(|| "角色包包含不安全路径".to_string())?;
        if enclosed.components().count() != 1 {
            return Err("角色包资源必须直接位于压缩包根目录".into());
        }
        let name = enclosed
            .file_name()
            .and_then(|value| value.to_str())
            .ok_or_else(|| "角色包包含无效文件名".to_string())?;
        if !allowed.contains(name) || !seen.insert(name.to_owned()) {
            return Err(format!("角色包包含不允许或重复的文件：{name}"));
        }
        if entry.size() > MAX_ARCHIVE_FILE_BYTES {
            return Err(format!("角色包文件过大：{name}"));
        }
        let expected_size = entry.size();
        total = total.saturating_add(expected_size);
        if total > MAX_ARCHIVE_BYTES {
            return Err("角色包解压后总大小超过限制".into());
        }
        if entry.compressed_size() > 0
            && entry.size() > 1024 * 1024
            && entry.size() / entry.compressed_size().max(1) > 200
        {
            return Err("角色包压缩比异常".into());
        }

        let output_path = destination.join(name);
        let mut output = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(output_path)
            .map_err(error_text)?;
        let copied = io::copy(&mut entry, &mut output).map_err(error_text)?;
        if copied != expected_size || copied > MAX_ARCHIVE_FILE_BYTES {
            return Err(format!("角色包文件实际大小异常：{name}"));
        }
        output.sync_all().map_err(error_text)?;
    }
    if !seen.contains("pet.json") {
        return Err("角色包根目录必须包含 pet.json".into());
    }
    Ok(())
}

fn broadcast_state(app: &AppHandle, state: &AppState) {
    if let Err(error) = app.emit("qpets://state-changed", state) {
        eprintln!("failed to broadcast state: {error}");
    }
}

fn apply_window_settings(app: &AppHandle, settings: &Settings) -> CommandResult<()> {
    let pet = app
        .get_webview_window("pet")
        .ok_or_else(|| "桌宠窗口不可用".to_string())?;
    pet.set_always_on_top(settings.always_on_top)
        .map_err(error_text)?;
    pet.set_ignore_cursor_events(settings.click_through)
        .map_err(error_text)?;
    pet.set_size(LogicalSize::new(
        PET_BASE_WIDTH * settings.scale,
        PET_BASE_HEIGHT * settings.scale,
    ))
    .map_err(error_text)
}

fn apply_autostart(app: &AppHandle, enabled: bool) -> CommandResult<()> {
    let manager = app.autolaunch();
    if enabled {
        manager.enable().map_err(error_text)
    } else {
        manager.disable().map_err(error_text)
    }
}

#[tauri::command]
fn get_app_state(store: State<'_, Mutex<Store>>) -> CommandResult<AppState> {
    Ok(store.lock().map_err(error_text)?.app_state())
}

#[tauri::command]
fn update_settings(
    settings: Settings,
    app: AppHandle,
    store: State<'_, Mutex<Store>>,
) -> CommandResult<AppState> {
    settings.validate()?;
    let mut store = store.lock().map_err(error_text)?;
    let previous = store.persisted.settings.clone();

    apply_autostart(&app, settings.start_with_windows)?;
    if let Err(error) = apply_window_settings(&app, &settings) {
        let _ = apply_autostart(&app, previous.start_with_windows);
        return Err(error);
    }

    store.persisted.settings = settings;
    if let Err(error) = store.save() {
        store.persisted.settings = previous.clone();
        let _ = apply_autostart(&app, previous.start_with_windows);
        let _ = apply_window_settings(&app, &previous);
        return Err(error);
    }
    let state = store.app_state();
    drop(store);
    broadcast_state(&app, &state);
    Ok(state)
}

#[tauri::command]
fn select_pet(
    pet_id: String,
    app: AppHandle,
    store: State<'_, Mutex<Store>>,
) -> CommandResult<AppState> {
    let mut store = store.lock().map_err(error_text)?;
    if !store.has_pet(&pet_id) {
        return Err("角色不存在或已损坏".into());
    }
    let previous = std::mem::replace(&mut store.persisted.selected_pet_id, pet_id);
    if let Err(error) = store.save() {
        store.persisted.selected_pet_id = previous;
        return Err(error);
    }
    let state = store.app_state();
    drop(store);
    broadcast_state(&app, &state);
    Ok(state)
}

#[tauri::command]
fn import_static_pet(
    source_path: String,
    name: Option<String>,
    app: AppHandle,
    store: State<'_, Mutex<Store>>,
) -> CommandResult<PetDescriptor> {
    let source = PathBuf::from(source_path);
    if !source.is_file() {
        return Err("选择的角色图片不存在".into());
    }
    let extension = source
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase();
    if !matches!(extension.as_str(), "png" | "webp") {
        return Err("静态角色仅支持 PNG 或 WebP".into());
    }

    let display_name = name
        .or_else(|| {
            source
                .file_stem()
                .and_then(|value| value.to_str())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| "自定义角色".into());
    let prefix = slug(&display_name);
    let id = format!(
        "user.{}.{}",
        if prefix.is_empty() { "pet" } else { &prefix },
        operation_id()
    );

    let expected_format = if extension == "png" {
        ImageFormat::Png
    } else {
        ImageFormat::WebP
    };
    let image = decode_image(&source, expected_format, 4096, 4096)?;
    require_transparent_image(&image)?;

    let mut store = store.lock().map_err(error_text)?;
    let staging = store.staging_dir().join(&id);
    fs::create_dir(&staging).map_err(error_text)?;
    let result = (|| {
        image
            .save_with_format(staging.join("character.png"), image::ImageFormat::Png)
            .map_err(error_text)?;
        generate_thumbnail(
            &image,
            RenderType::StaticImageV1,
            &staging.join("thumbnail.png"),
        )?;
        write_json_file(
            &staging.join("dialogues.json"),
            &default_dialogues(&display_name),
        )?;
        write_json_file(
            &staging.join("pet.json"),
            &PetManifest {
                schema_version: 1,
                id,
                display_name,
                description: "用户添加的静态桌宠。".into(),
                render_type: RenderType::StaticImageV1,
                sprite_version_number: None,
                spritesheet_path: None,
                character_path: Some("character.png".into()),
                thumbnail_path: "thumbnail.png".into(),
                dialogues_path: "dialogues.json".into(),
            },
        )?;
        install_staged(&mut store, &staging)
    })();

    let descriptor = match result {
        Ok(descriptor) => descriptor,
        Err(error) => {
            let _ = fs::remove_dir_all(&staging);
            return Err(error);
        }
    };
    let state = store.app_state();
    drop(store);
    broadcast_state(&app, &state);
    Ok(descriptor)
}

#[tauri::command]
fn import_pet_package(
    source_path: String,
    _name: Option<String>,
    app: AppHandle,
    store: State<'_, Mutex<Store>>,
) -> CommandResult<PetDescriptor> {
    let source = PathBuf::from(source_path);
    if !source.is_file() {
        return Err("选择的角色包不存在".into());
    }
    let mut store = store.lock().map_err(error_text)?;
    let staging = store
        .staging_dir()
        .join(format!("import-{}", operation_id()));
    let result = (|| {
        extract_qpet(&source, &staging)?;
        install_staged(&mut store, &staging)
    })();
    let descriptor = match result {
        Ok(descriptor) => descriptor,
        Err(error) => {
            let _ = fs::remove_dir_all(&staging);
            return Err(error);
        }
    };
    let state = store.app_state();
    drop(store);
    broadcast_state(&app, &state);
    Ok(descriptor)
}

#[tauri::command]
fn delete_pet(
    pet_id: String,
    app: AppHandle,
    store: State<'_, Mutex<Store>>,
) -> CommandResult<AppState> {
    let mut store = store.lock().map_err(error_text)?;
    if store.builtins.iter().any(|pet| pet.id == pet_id) {
        return Err("内置角色受保护，不能删除".into());
    }
    if !is_valid_pet_id(&pet_id, false) {
        return Err("自定义角色 ID 无效".into());
    }
    let index = store
        .persisted
        .user_pets
        .iter()
        .position(|record| record.id == pet_id)
        .ok_or_else(|| "自定义角色不存在".to_string())?;

    let source = store.package_dir(&pet_id)?;
    let trash = store
        .trash_dir()
        .join(format!("{}-{}", pet_id, operation_id()));
    fs::rename(&source, &trash).map_err(error_text)?;

    let removed = store.persisted.user_pets.remove(index);
    let previous_selected = store.persisted.selected_pet_id.clone();
    if previous_selected == pet_id {
        store.persisted.selected_pet_id = DEFAULT_PET_ID.into();
    }
    if let Err(error) = store.save() {
        store.persisted.user_pets.insert(index, removed);
        store.persisted.selected_pet_id = previous_selected;
        let _ = fs::rename(&trash, &source);
        return Err(error);
    }
    if let Err(error) = fs::remove_dir_all(&trash) {
        eprintln!("failed to clean pet trash {}: {error}", trash.display());
    }
    let state = store.app_state();
    drop(store);
    broadcast_state(&app, &state);
    Ok(state)
}

fn show_settings(window: &WebviewWindow) -> CommandResult<()> {
    window.show().map_err(error_text)?;
    window.set_focus().map_err(error_text)
}

#[tauri::command]
fn open_settings(app: AppHandle) -> CommandResult<()> {
    show_settings(
        &app.get_webview_window("settings")
            .ok_or_else(|| "设置窗口不可用".to_string())?,
    )
}

#[tauri::command]
fn toggle_pet(app: AppHandle) -> CommandResult<bool> {
    let pet = app
        .get_webview_window("pet")
        .ok_or_else(|| "桌宠窗口不可用".to_string())?;
    if pet.is_visible().map_err(error_text)? {
        pet.hide().map_err(error_text)?;
        Ok(false)
    } else {
        pet.show().map_err(error_text)?;
        Ok(true)
    }
}

fn hide_instead_of_close(window: &WebviewWindow) {
    let window = window.clone();
    window.clone().on_window_event(move |event| {
        if let tauri::WindowEvent::CloseRequested { api, .. } = event {
            api.prevent_close();
            let _ = window.hide();
        }
    });
}

fn prepare_dev_builtins(source: &Path, destination: &Path) -> CommandResult<()> {
    let parent = destination
        .parent()
        .ok_or_else(|| "开发资源目标路径无效".to_string())?;
    fs::create_dir_all(parent).map_err(error_text)?;
    let staging = parent.join(format!(".builtin-{}", operation_id()));
    fs::create_dir(&staging).map_err(error_text)?;

    let result = (|| {
        let index: PetIndex = read_json(&source.join("index.json"))?;
        fs::copy(source.join("index.json"), staging.join("index.json")).map_err(error_text)?;
        for entry in index.pets {
            if !is_safe_file_name(&entry.package_path)
                || !is_valid_pet_id(&entry.id, true)
                || entry.id != entry.package_path
            {
                return Err("开发资源索引包含无效包路径".into());
            }
            let source_package = source.join(&entry.package_path);
            let target_package = staging.join(&entry.package_path);
            fs::create_dir(&target_package).map_err(error_text)?;
            for name in [
                "pet.json",
                "dialogues.json",
                "thumbnail.png",
                "spritesheet.webp",
                "character.png",
            ] {
                let source_file = source_package.join(name);
                if source_file.is_file() {
                    fs::copy(&source_file, target_package.join(name)).map_err(error_text)?;
                }
            }
        }
        if destination.exists() {
            fs::remove_dir_all(destination).map_err(error_text)?;
        }
        fs::rename(&staging, destination).map_err(error_text)
    })();

    if result.is_err() {
        let _ = fs::remove_dir_all(staging);
    }
    result
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_dialog::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            None,
        ))
        .setup(|app| {
            let root = app.path().app_data_dir()?;
            let bundled = app.path().resource_dir()?.join("resources").join("pets");
            let resource_pets = if bundled.is_dir() {
                bundled
            } else {
                let source = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
                    .join("resources")
                    .join("pets");
                let destination = root.join(PETS_DIR).join("builtin");
                prepare_dev_builtins(&source, &destination).map_err(io::Error::other)?;
                destination
            };
            let store = Store::open(root, resource_pets).map_err(io::Error::other)?;
            apply_window_settings(app.handle(), &store.persisted.settings)
                .map_err(io::Error::other)?;
            if let Err(error) =
                apply_autostart(app.handle(), store.persisted.settings.start_with_windows)
            {
                eprintln!("failed to synchronize autostart: {error}");
            }
            app.manage(Mutex::new(store));

            let settings = MenuItem::with_id(app, "settings", "设置", true, None::<&str>)?;
            let toggle = MenuItem::with_id(app, "toggle-pet", "显示/隐藏桌宠", true, None::<&str>)?;
            let quit = MenuItem::with_id(app, "quit", "退出", true, None::<&str>)?;
            let menu = Menu::with_items(app, &[&settings, &toggle, &quit])?;
            let mut tray = TrayIconBuilder::new().menu(&menu).tooltip("Qpets");
            if let Some(icon) = app.default_window_icon() {
                tray = tray.icon(icon.clone());
            }
            tray.on_menu_event(|app, event| match event.id.as_ref() {
                "settings" => {
                    let _ = open_settings(app.clone());
                }
                "toggle-pet" => {
                    let _ = toggle_pet(app.clone());
                }
                "quit" => app.exit(0),
                _ => {}
            })
            .build(app)?;

            for label in ["pet", "settings"] {
                let window = app.get_webview_window(label).ok_or_else(|| {
                    io::Error::new(io::ErrorKind::NotFound, format!("missing window {label}"))
                })?;
                hide_instead_of_close(&window);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_app_state,
            update_settings,
            select_pet,
            import_static_pet,
            import_pet_package,
            delete_pet,
            open_settings,
            toggle_pet
        ])
        .run(tauri::generate_context!())
        .expect("error while running Qpets");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pet_ids_are_scoped_and_portable() {
        assert!(is_valid_pet_id("user.sunny-01", false));
        assert!(is_valid_pet_id("qpet-built-in", true));
        assert!(!is_valid_pet_id("qpet-built-in", false));
        assert!(!is_valid_pet_id("qpet.built-in", false));
        assert!(!is_valid_pet_id("user.sunny-", false));
        assert!(!is_valid_pet_id("UpperCase", false));
        assert!(!is_valid_pet_id("../escape", false));
    }

    #[test]
    fn package_paths_stay_at_the_root() {
        assert!(is_safe_file_name("spritesheet.webp"));
        assert!(!is_safe_file_name("../spritesheet.webp"));
        assert!(!is_safe_file_name("nested/spritesheet.webp"));
    }

    #[test]
    fn settings_reject_out_of_range_values() {
        let mut settings = Settings::default();
        settings.scale = 2.0;
        assert!(settings.validate().is_err());
    }
}
