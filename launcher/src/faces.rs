//! The picture the launcher shows for a player: one of its built-in pixel faces, or a picture the
//! player uploaded (PNG, WebP or JPEG, transparency kept). Choices live in `faces.json` under the
//! data folder, one per player, so the picture follows whoever plays; uploaded files are copied
//! into `faces/` and read again at start.
use super::*;

/// A built-in face: an 8×8 pixel drawing, `#` in `ink`, `o` in `shade`, `.` transparent.
struct BuiltIn {
    art: [&'static str; 8],
    ink: u32,
    shade: u32,
}

const BUILT_IN: [BuiltIn; 8] = [
    // A smiley.
    BuiltIn {
        art: [
            "..####..", ".######.", "##o##o##", "########", "##oooo##", "#o####o#", ".######.",
            "..####..",
        ],
        ink: 0xd0a05a,
        shade: 0x6a4a28,
    },
    // A creeper.
    BuiltIn {
        art: [
            "########", "#oo##oo#", "#oo##oo#", "########", "###oo###", "##oooo##", "##oooo##",
            "########",
        ],
        ink: 0x59b35c,
        shade: 0x1d4d20,
    },
    // A robot.
    BuiltIn {
        art: [
            "..oooo..", ".######.", "#o####o#", "#o####o#", "########", "#o#oo#o#", ".######.",
            "..oooo..",
        ],
        ink: 0x8a93a8,
        shade: 0x2b3040,
    },
    // A cat.
    BuiltIn {
        art: [
            "#......#", "##....##", "########", "#o####o#", "###oo###", "#o####o#", "##oooo##",
            ".######.",
        ],
        ink: 0xd08b2c,
        shade: 0x4a2b0a,
    },
    // A slime.
    BuiltIn {
        art: [
            "..####..", ".######.", "##o##o##", "########", "########", "#oooooo#", ".######.",
            "..####..",
        ],
        ink: 0x59c47a,
        shade: 0x14512a,
    },
    // A pumpkin.
    BuiltIn {
        art: [
            "....oo..", "########", "#oo##oo#", "########", "#o####o#", "##oooo##", "########",
            "..####..",
        ],
        ink: 0xe08a2e,
        shade: 0x5a2c05,
    },
    // A ghost.
    BuiltIn {
        art: [
            "..####..", ".######.", "##o##o##", "########", "########", "########", "########",
            "#.#.#.#.",
        ],
        ink: 0xc9c6d6,
        shade: 0x4a4658,
    },
    // An underwater mask.
    BuiltIn {
        art: [
            "..####..", ".######.", "#o####o#", "########", "#oooooo#", "##oooo##", ".######.",
            "..####..",
        ],
        ink: 0x2f9ec4,
        shade: 0x0c3b4c,
    },
];

/// How many built-in faces there are.
pub const COUNT: usize = BUILT_IN.len();

fn colour(rgb: u32) -> [u8; 4] {
    [(rgb >> 16) as u8, (rgb >> 8) as u8, rgb as u8, 255]
}

/// A built-in face's texels, side by side (`COUNT` of them).
pub fn built_in(index: usize) -> Option<(Vec<[u8; 4]>, usize)> {
    let face = BUILT_IN.get(index)?;
    let (ink, shade) = (colour(face.ink), colour(face.shade));
    let mut texels = Vec::with_capacity(64);
    for row in face.art {
        for c in row.chars() {
            texels.push(match c {
                '#' => ink,
                'o' => shade,
                _ => [0, 0, 0, 0],
            });
        }
    }
    Some((texels, 8))
}

/// `faces.json`: the choice kept for each player's key.
fn file() -> std::path::PathBuf {
    backend::data_dir().join("faces.json")
}

/// The pictures kept for each player.
pub fn stored() -> HashMap<String, String> {
    std::fs::read(file())
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
        .unwrap_or_default()
}

/// Keeps `choice` for the player `key`, or forgets their choice.
pub fn choose(key: &str, choice: Option<String>) -> Result<(), String> {
    let mut all = stored();
    match choice {
        Some(choice) => {
            all.insert(key.to_string(), choice);
        }
        None => {
            all.remove(key);
        }
    }
    keep(&all)
}

fn keep(all: &HashMap<String, String>) -> Result<(), String> {
    std::fs::create_dir_all(backend::data_dir()).map_err(|e| e.to_string())?;
    std::fs::write(
        file(),
        serde_json::to_vec_pretty(all).map_err(|e| e.to_string())?,
    )
    .map_err(|e| e.to_string())
}

/// The key a player's picture is kept under: a Microsoft account's UUID, or an offline name's
/// offline UUID (the same key its head is kept under).
pub fn key(player: &account::Player, name: &str) -> Option<String> {
    match player {
        account::Player::Microsoft(uuid) => Some(uuid.clone()),
        account::Player::Offline(name) => Some(profile_heads::offline_key(name)),
        account::Player::GameName => Some(profile_heads::offline_key(name)),
    }
}

/// A choice's texels: `face:<n>` one of [`BUILT_IN`], `image:<file>` a picture kept in `faces/`
/// of `dir`.
pub fn decode_in(dir: &std::path::Path, choice: &str) -> Option<(Vec<[u8; 4]>, usize)> {
    if let Some(index) = choice.strip_prefix("face:") {
        return built_in(index.parse().ok()?);
    }
    let file = choice.strip_prefix("image:")?;
    let bytes = std::fs::read(dir.join("faces").join(file)).ok()?;
    let image = image::load_from_memory(&bytes).ok()?.to_rgba8();
    const SIDE: u32 = 32;
    let image = image::imageops::resize(
        &image,
        SIDE,
        SIDE,
        image::imageops::FilterType::Triangle,
    );
    Some((image.pixels().map(|p| p.0).collect(), SIDE as usize))
}

fn safe(key: &str) -> String {
    key.chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect()
}

pub fn decode(choice: &str) -> Option<(Vec<[u8; 4]>, usize)> {
    decode_in(&backend::data_dir(), choice)
}

/// Copies a picture of the player's into `faces/` of `dir`, as PNG, and answers the choice to
/// keep. PNG, WebP and JPEG are read; transparency survives.
pub fn upload_in(
    dir: &std::path::Path,
    key: &str,
    path: &std::path::Path,
) -> Result<String, String> {
    let bytes = std::fs::read(path).map_err(|e| e.to_string())?;
    let image = image::load_from_memory(&bytes)
        .map_err(|_| {
            tr("That file is not a picture the launcher can read (PNG, WebP or JPEG).").to_string()
        })?
        .to_rgba8();
    let dir = dir.join("faces");
    std::fs::create_dir_all(&dir).map_err(|e| e.to_string())?;
    let name = format!("{}.png", safe(key));
    const SIDE: u32 = 64;
    let small = image::imageops::resize(&image, SIDE, SIDE, image::imageops::FilterType::Triangle);
    small
        .save(dir.join(&name))
        .map_err(|e| e.to_string())?;
    Ok(format!("image:{name}"))
}

pub fn upload(key: &str, path: &std::path::Path) -> Result<String, String> {
    upload_in(&backend::data_dir(), key, path)
}

/// Moves the picture kept for one player's key to another's: a renamed offline name keeps its
/// icon, and an uploaded picture's file goes with the new key.
pub fn rename(from: &str, to: &str) {
    let mut all = stored();
    let Some(choice) = all.remove(from) else {
        return;
    };
    let choice = match choice.strip_prefix("image:") {
        Some(file) => {
            let dir = backend::data_dir().join("faces");
            let name = format!("{}.png", safe(to));
            let _ = std::fs::rename(dir.join(file), dir.join(&name));
            format!("image:{name}")
        }
        None => choice,
    };
    all.insert(to.to_string(), choice);
    let _ = keep(&all);
}

impl Launcher {
    /// Reads the picture kept for every player into the head cache: whoever plays shows theirs.
    pub(super) fn load_faces(&mut self) {
        for (key, choice) in &self.faces {
            self.profile_heads.set_custom(key, decode(choice));
        }
    }

    /// The picture kept for whoever plays now.
    pub(super) fn face_choice(&self) -> Option<String> {
        let key = self.face_key()?;
        self.faces.get(&key).cloned()
    }

    pub(super) fn face_key(&self) -> Option<String> {
        let (name, _) = self.who_plays();
        key(&self.accounts.player(), &name)
    }

    /// Shows the player a built-in face (`Some("face:n")`), a picture they picked
    /// (`Some("image:…")`), or their own skin again (`None`), and keeps the choice.
    pub(super) fn set_face(&mut self, choice: Option<String>, cx: &mut Context<Self>) {
        let Some(key) = self.face_key() else {
            return;
        };
        if let Err(e) = choose(&key, choice.clone()) {
            crate::glass_toast::say(SharedString::from(e));
            return;
        }
        self.faces = stored();
        self.profile_heads
            .set_custom(&key, choice.as_deref().and_then(decode));
        self.faces_menu = false;
        cx.notify();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn scratch(name: &str) -> std::path::PathBuf {
        let dir = std::env::temp_dir().join(format!("hrizn-faces-{}-{name}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// A 4×4 picture: a solid green middle, every other pixel clear.
    fn pixels(alpha: bool) -> Vec<u8> {
        let mut raw = Vec::with_capacity(4 * 4 * 4);
        for y in 0..4 {
            for x in 0..4 {
                let solid = (1..3).contains(&x) && (1..3).contains(&y);
                let a = if solid || !alpha { 255 } else { 0 };
                raw.extend_from_slice(&[30, 200, 90, a]);
            }
        }
        raw
    }

    fn encoded(format: image::ImageFormat) -> Vec<u8> {
        use image::ImageEncoder;
        let mut bytes = Vec::new();
        let (side, kind) = (4u32, image::ExtendedColorType::Rgba8);
        match format {
            image::ImageFormat::Png => {
                image::codecs::png::PngEncoder::new(&mut bytes)
                    .write_image(&pixels(true), side, side, kind)
                    .unwrap();
            }
            image::ImageFormat::WebP => {
                image::codecs::webp::WebPEncoder::new_lossless(&mut bytes)
                    .write_image(&pixels(true), side, side, kind)
                    .unwrap();
            }
            _ => {
                // JPEG keeps no transparency, so the picture is encoded opaque, without one.
                let rgb: Vec<u8> = pixels(false)
                    .chunks(4)
                    .flat_map(|px| px[..3].to_vec())
                    .collect();
                image::codecs::jpeg::JpegEncoder::new_with_quality(&mut bytes, 92)
                    .write_image(&rgb, side, side, image::ExtendedColorType::Rgb8)
                    .unwrap();
            }
        }
        bytes
    }

    #[test]
    fn an_uploaded_picture_is_read_in_every_supported_format() {
        for (extension, format) in [
            ("png", image::ImageFormat::Png),
            ("webp", image::ImageFormat::WebP),
            ("jpg", image::ImageFormat::Jpeg),
        ] {
            let dir = scratch(extension);
            let file = dir.join(format!("picture.{extension}"));
            std::fs::write(&file, encoded(format)).unwrap();
            let choice = upload_in(&dir, "player:one", &file)
                .unwrap_or_else(|e| panic!("{extension} upload failed: {e}"));
            assert!(choice.starts_with("image:"), "{choice}");
            let (texels, side) = decode_in(&dir, &choice)
                .unwrap_or_else(|| panic!("{extension} decodes"));
            assert_eq!((side, texels.len()), (32, 32 * 32));
            let alpha = |x: usize, y: usize| texels[y * 32 + x][3];
            assert!(alpha(16, 16) > 200, "{extension} keeps its middle");
            if format != image::ImageFormat::Jpeg {
                assert!(alpha(1, 1) < 64, "{extension} keeps transparency");
            }
        }
    }

    #[test]
    fn the_built_in_faces_are_eight_by_eight() {
        for index in 0..COUNT {
            let (texels, side) = built_in(index).unwrap();
            assert_eq!((side, texels.len()), (8, 64));
            assert!(
                texels.iter().all(|t| t[3] == 0 || t[3] == 255),
                "face {index} is solid or clear, never half-drawn"
            );
        }
        assert!(built_in(COUNT).is_none());
        assert_eq!(decode_in(std::path::Path::new("nowhere"), "face:0").unwrap().1, 8);
        assert!(decode_in(std::path::Path::new("nowhere"), "image:gone.png").is_none());
    }
}
