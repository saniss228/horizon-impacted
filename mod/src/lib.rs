//! Independent in-game interface for the modified Horizon Native client. Not the Java Impact client.
//! Built against the Horizon v0.2.92 WebAssembly SDK; unsupported modules are
//! deliberately not represented as functional switches.
use horizon_mod_sdk::{
    Mod, Registration, chat,
    game::{self, Clock, Player},
    hud::{self, Corner as HudCorner, Line},
    input::{self, Event},
    render::{self, Corner as MapCorner, FillShape, Minimap, Rect, Shape, Zoom},
    storage,
    ui::{self, ButtonWidget, Screen, TextWidget, Widget},
};

const MENU_KEY: &str = "impact_menu";
const ZOOM_KEY: &str = "impact_zoom";
const ID_BACK: u32 = 1;
const ID_DISPLAY: u32 = 2;
const ID_WAYPOINTS: u32 = 3;
const ID_STATS: u32 = 4;
const ID_TOOLS: u32 = 5;
const ID_POINTS_MORE: u32 = 6;
const ID_COORDS: u32 = 10;
const ID_FACING: u32 = 11;
const ID_CLOCK: u32 = 12;
const ID_HEALTH: u32 = 13;
const ID_FPS: u32 = 14;
const ID_CPS: u32 = 15;
const ID_SPEED: u32 = 16;
const ID_MAP: u32 = 17;
const ID_ZOOM: u32 = 18;
const ID_WAYPOINT: u32 = 19;
const ID_DEATH: u32 = 20;
const ID_SAVE_POINT: u32 = 40;
const ID_CLEAR_POINT: u32 = 41;
const ID_CLEAR_DEATH: u32 = 42;
const ID_HUD: u32 = 50;
const ID_MODULES: u32 = 51;
const ID_MODULES_MORE: u32 = 52;
const ID_ORE: u32 = 53;
const ID_ATTACK: u32 = 54;
const ID_WALK: u32 = 55;
const ID_RADAR: u32 = 56;
const ID_STOP_ALL: u32 = 57;
const ID_MAP_OPTIONS: u32 = 60;
const ID_MAP_MORE: u32 = 61;
const ID_MAP_CORNER: u32 = 62;
const ID_MAP_SIZE: u32 = 63;
const ID_ARC: u32 = 64;
const ID_VISUALS: u32 = 65;
const ID_ORE_ESP: u32 = 66;
const ID_MOB_ESP: u32 = 67;
const ID_SEARCH_SETTINGS: u32 = 68;
const ID_ORE_RADIUS: u32 = 69;
const ID_AIM_RADIUS: u32 = 79;
const ID_AIM_WALLS: u32 = 80;
const ID_ENTITY_GLOW: u32 = 81;
const ID_XRAY: u32 = 82;
const ID_ORE_TYPE: u32 = 83;
const ID_GO_POINT: u32 = 84;
const ID_MENU_BIND: u32 = 85;
const ID_PLAYER_ATTACK: u32 = 86;
const ID_PLAYER_BOW: u32 = 87;
const ID_CRITS: u32 = 88;
const ID_FRIENDS: u32 = 89;
const ID_CLEAR_FRIENDS: u32 = 90;
const ID_BRIDGE_SELECT: u32 = 91;
const ID_TRACERS: u32 = 92;
const ID_LIST_FRIENDS: u32 = 93;
const ID_XRAY_BASE: u32 = 110;
const XRAY_ALL: u32 = (1 << 13) - 1;
const XRAY_NAMES: [(&str, &str); 13] = [
    ("diamond", "Diamond"),
    ("iron", "Iron"),
    ("gold", "Gold"),
    ("redstone", "Redstone"),
    ("emerald", "Emerald"),
    ("coal", "Coal"),
    ("copper", "Copper"),
    ("lapis", "Lapis"),
    ("quartz", "Quartz"),
    ("debris", "Debris"),
    ("water", "Water"),
    ("lava", "Lava"),
    ("other", "Other ore"),
];
const ID_MODULES_ADVANCED: u32 = 70;
const ID_MODULES_AIM: u32 = 71;
const ID_COMBAT_SETTINGS: u32 = 72;
const ID_MINE: u32 = 73;
const ID_AIM: u32 = 74;
const ID_BOW: u32 = 75;
const ID_RANGE: u32 = 76;
const ID_DELAY: u32 = 77;
const ID_TARGETS: u32 = 78;
const NATIVE_REQUEST: &str = "native_request";
const NATIVE_STATUS: &str = "native_status";
const NATIVE_VISUAL: &str = "native_visual";
const BRIDGE_STATUS: &str = "bridge_status";
const BRIDGE_REQUEST: &str = "bridge_request";
const BRIDGE_RADAR: &str = "bridge_radar";
const BRIDGE_STALE_SECONDS: f32 = 4.0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum Page {
    Home,
    Hud,
    Modules,
    ModulesMore,
    ModulesAdvanced,
    ModulesAim,
    CombatSettings,
    Display,
    Stats,
    Tools,
    MapOptions,
    MapMore,
    Visuals,
    SearchSettings,
    Waypoints,
    PointsMore,
}

#[derive(Debug)]
struct Settings {
    coords: bool,
    facing: bool,
    clock: bool,
    health: bool,
    fps: bool,
    cps: bool,
    speed: bool,
    map: bool,
    map_corner: u8,
    map_size: u16,
    arc: bool,
    ore_esp: bool,
    mob_esp: bool,
    entity_glow: bool,
    xray: bool,
    aim_walls: bool,
    tracers: bool,
    xray_mask: u32,
    xray_rules: Vec<(String, bool)>,
    zoom: bool,
    waypoint: bool,
    death: bool,
}

impl Settings {
    fn load() -> Self {
        Self {
            coords: stored_flag("coords", true),
            facing: stored_flag("facing", true),
            clock: stored_flag("clock", false),
            health: stored_flag("health", false),
            fps: stored_flag("fps", false),
            cps: stored_flag("cps", false),
            speed: stored_flag("speed", false),
            map: stored_flag("map", false),
            map_corner: stored("map_corner")
                .and_then(|v| v.parse::<u8>().ok())
                .filter(|n| *n < 4)
                .unwrap_or(0),
            map_size: stored("map_size")
                .and_then(|v| v.parse::<u16>().ok())
                .filter(|n| [96, 192, 256].contains(n))
                .unwrap_or(96),
            arc: stored_flag("arc", false),
            ore_esp: stored_flag("espore", false),
            mob_esp: stored_flag("espmobs", false),
            entity_glow: stored_flag("glow", false),
            xray: false, // never unexpectedly hide terrain after restarting
            aim_walls: stored_flag("aimwalls", false),
            tracers: stored_flag("tracers", false),
            xray_mask: stored("xray_mask")
                .and_then(|v| v.parse::<u32>().ok())
                .filter(|v| *v <= XRAY_ALL)
                .unwrap_or(XRAY_ALL),
            xray_rules: stored("xray_rules")
                .and_then(|v| parse_xray_rules(&v))
                .unwrap_or_default(),
            zoom: stored_flag("zoom", true),
            waypoint: stored_flag("waypoint", true),
            death: stored_flag("death", false),
        }
    }
    fn flag_mut(&mut self, name: &str) -> Option<&mut bool> {
        match name {
            "coords" => Some(&mut self.coords),
            "facing" => Some(&mut self.facing),
            "clock" => Some(&mut self.clock),
            "health" => Some(&mut self.health),
            "fps" => Some(&mut self.fps),
            "cps" => Some(&mut self.cps),
            "speed" => Some(&mut self.speed),
            "map" => Some(&mut self.map),
            "arc" => Some(&mut self.arc),
            "espore" => Some(&mut self.ore_esp),
            "espmobs" => Some(&mut self.mob_esp),
            "glow" => Some(&mut self.entity_glow),
            "xray" => Some(&mut self.xray),
            "aimwalls" => Some(&mut self.aim_walls),
            "tracers" => Some(&mut self.tracers),
            "zoom" => Some(&mut self.zoom),
            "waypoint" => Some(&mut self.waypoint),
            "death" => Some(&mut self.death),
            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
struct Point {
    x: f64,
    y: f64,
    z: f64,
    dimension: String,
}
impl Point {
    fn from_player(p: &Player) -> Self {
        Self {
            x: p.x,
            y: p.y,
            z: p.z,
            dimension: p.dimension.clone(),
        }
    }
    fn encode(&self) -> String {
        format!("{}|{}|{}|{}", self.x, self.y, self.z, self.dimension)
    }
    fn decode(text: &str) -> Option<Self> {
        let mut parts = text.split('|');
        let x: f64 = parts.next()?.parse().ok()?;
        let y: f64 = parts.next()?.parse().ok()?;
        let z: f64 = parts.next()?.parse().ok()?;
        let dimension = parts.next()?.to_owned();
        if parts.next().is_some()
            || !x.is_finite()
            || !y.is_finite()
            || !z.is_finite()
            || dimension.is_empty()
            || dimension.len() > 128
        {
            return None;
        }
        Some(Self { x, y, z, dimension })
    }
    fn short(&self) -> String {
        format!(
            "{:.0} / {:.0} / {:.0}",
            self.x.floor(),
            self.y.floor(),
            self.z.floor()
        )
    }
    fn distance(&self, p: &Player) -> Option<f64> {
        if self.dimension != p.dimension {
            return None;
        }
        Some(((self.x - p.x).powi(2) + (self.y - p.y).powi(2) + (self.z - p.z).powi(2)).sqrt())
    }
}

fn stored(key: &str) -> Option<String> {
    storage::get(key).and_then(|v| String::from_utf8(v).ok())
}
fn parse_flag(text: Option<&str>, default: bool) -> bool {
    match text.map(str::trim) {
        Some("true") => true,
        Some("false") => false,
        _ => default,
    }
}
fn stored_flag(key: &str, default: bool) -> bool {
    parse_flag(stored(key).as_deref(), default)
}
fn save_flag(key: &str, value: bool) -> Result<(), String> {
    let bytes: &[u8] = if value { b"true" } else { b"false" };
    storage::set(key, bytes).map_err(|e| format!("Cannot save {key}: {e}"))
}
fn read_point(key: &str) -> Option<Point> {
    stored(key).and_then(|v| Point::decode(&v))
}
fn save_point(key: &str, p: &Point) -> Result<(), String> {
    storage::set(key, p.encode().as_bytes()).map_err(|e| format!("Cannot save {key}: {e}"))
}

fn facing(yaw: f32) -> &'static str {
    let i = (yaw / 90.0 + 0.5).floor() as i32 & 3;
    ["South (+Z)", "West (-X)", "North (-Z)", "East (+X)"][i as usize]
}
fn time_of_day(c: &Clock) -> String {
    let day = c.day_time / 24000 + 1;
    let ticks = c.day_time % 24000;
    let hours = (ticks / 1000 + 6) % 24;
    let minutes = ticks % 1000 * 60 / 1000;
    format!("Day {day}, {hours:02}:{minutes:02}")
}
fn button(id: u32, label: String) -> Widget {
    Widget::Button(ButtonWidget { id, label })
}
fn on_off(on: bool) -> &'static str {
    if on { "ON" } else { "OFF" }
}

/// Volatile status sent by the modified Horizon host to this exact mod. In the
/// native build these reserved storage keys live only in the same process.
#[derive(Clone, Debug, PartialEq)]
struct BridgeStatus {
    session: String,
    heartbeat: u64,
    connected: bool,
    ore: bool,
    attack: bool,
    walk: bool,
    radar: bool,
    ore_kind: String,
    dimension: String,
    ore_pos: [i32; 3],
    radar_count: u16,
    radar_kind: String,
    radar_pos: [i32; 3],
}
fn bridge_name(s: &str, max: usize) -> bool {
    !s.is_empty()
        && s.len() <= max
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_:-".contains(&c))
}
fn bridge_dimension(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 80
        && s.bytes()
            .all(|c| c.is_ascii_alphanumeric() || b"_:-/.".contains(&c))
}
fn bridge_bool(s: &str) -> Option<bool> {
    match s {
        "0" => Some(false),
        "1" => Some(true),
        _ => None,
    }
}
impl BridgeStatus {
    fn decode(s: &str) -> Option<Self> {
        if s.len() > 400 || !s.is_ascii() {
            return None;
        }
        let a: Vec<&str> = s.trim().split('|').collect();
        if a.len() != 18
            || a[0] != "v1"
            || a[1].len() != 32
            || !a[1]
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
            || !bridge_name(a[8], 40)
            || !bridge_dimension(a[9])
            || !bridge_name(a[14], 48)
        {
            return None;
        }
        let result = Self {
            session: a[1].into(),
            heartbeat: a[2].parse().ok()?,
            connected: bridge_bool(a[3])?,
            ore: bridge_bool(a[4])?,
            attack: bridge_bool(a[5])?,
            walk: bridge_bool(a[6])?,
            radar: bridge_bool(a[7])?,
            ore_kind: a[8].into(),
            dimension: a[9].into(),
            ore_pos: [
                a[10].parse().ok()?,
                a[11].parse().ok()?,
                a[12].parse().ok()?,
            ],
            radar_count: a[13].parse().ok()?,
            radar_kind: a[14].into(),
            radar_pos: [
                a[15].parse().ok()?,
                a[16].parse().ok()?,
                a[17].parse().ok()?,
            ],
        };
        if result.radar_count > 4096 {
            return None;
        }
        Some(result)
    }
    fn request(&self, seq: u64, name: &str, value: bool) -> String {
        format!("v1|{}|{}|{}|{}", self.session, seq, name, u8::from(value))
    }
}

#[derive(Clone, Debug, PartialEq)]
struct NativeStatus {
    aim: bool,
    bow: bool,
    mine: bool,
    range: f32,
    delay: f32,
    all_mobs: bool,
    ore: String,
    radius: u8,
    nav: bool,
    aim_radius: f32,
    player_attack: bool,
    player_bow: bool,
    crits: bool,
    friends: u32,
}
impl NativeStatus {
    fn decode(text: &str) -> Option<Self> {
        if !text.is_ascii() || text.len() > 256 {
            return None;
        }
        let f: Vec<_> = text.trim().split('|').collect();
        if !matches!(f.len(), 10 | 11 | 15) || f[0] != "v1" {
            return None;
        }
        let range: f32 = f[4].parse().ok()?;
        let delay: f32 = f[5].parse().ok()?;
        let radius: u8 = f[8].parse().ok()?;
        let aim_radius: f32 = if f.len() >= 11 {
            f[10].parse().ok()?
        } else {
            48.0
        };
        if !aim_radius.is_finite()
            || !(4.0..=128.0).contains(&aim_radius)
            || !range.is_finite()
            || !(1.0..=6.0).contains(&range)
            || !delay.is_finite()
            || !(0.1..=5.0).contains(&delay)
            || !(4..=128).contains(&radius)
            || !bridge_name(f[7], 40)
        {
            return None;
        }
        let all_mobs = match f[6] {
            "all" => true,
            "hostile" => false,
            _ => return None,
        };
        Some(Self {
            aim: bridge_bool(f[1])?,
            bow: bridge_bool(f[2])?,
            mine: bridge_bool(f[3])?,
            range,
            delay,
            all_mobs,
            ore: f[7].into(),
            radius,
            nav: bridge_bool(f[9])?,
            aim_radius,
            player_attack: if f.len() == 15 {
                bridge_bool(f[11])?
            } else {
                false
            },
            player_bow: if f.len() == 15 {
                bridge_bool(f[12])?
            } else {
                false
            },
            crits: if f.len() == 15 {
                bridge_bool(f[13])?
            } else {
                false
            },
            friends: if f.len() == 15 {
                f[14].parse::<u32>().ok()?.min(512)
            } else {
                0
            },
        })
    }
}

#[derive(Clone, Debug, PartialEq)]
struct RadarData {
    session: String,
    dimension: String,
    effects: bool,
    markers: Vec<(f64, f64, bool)>,
}
impl RadarData {
    fn decode(text: &str) -> Option<Self> {
        if text.len() > 4096 || !text.is_ascii() {
            return None;
        }
        let fields: Vec<_> = text.trim().split('|').collect();
        if fields.len() != 5
            || fields[0] != "v1"
            || fields[1].len() != 32
            || !fields[1]
                .bytes()
                .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
            || !bridge_dimension(fields[2])
        {
            return None;
        }
        let mut markers = Vec::new();
        if !fields[4].is_empty() {
            for token in fields[4].split(';') {
                if markers.len() >= 32 {
                    return None;
                }
                let values: Vec<_> = token.split(',').collect();
                if values.len() != 3 {
                    return None;
                }
                let x: f64 = values[0].parse().ok()?;
                let z: f64 = values[1].parse().ok()?;
                if !x.is_finite()
                    || !z.is_finite()
                    || x.abs() > 30_000_000.0
                    || z.abs() > 30_000_000.0
                {
                    return None;
                }
                markers.push((x, z, bridge_bool(values[2])?));
            }
        }
        Some(Self {
            session: fields[1].into(),
            dimension: fields[2].into(),
            effects: bridge_bool(fields[3])?,
            markers,
        })
    }
}

fn corner_name(n: u8) -> &'static str {
    ["top right", "bottom right", "bottom left", "top left"][usize::from(n.min(3))]
}
fn map_corner(n: u8) -> MapCorner {
    match n {
        0 => MapCorner::TopRight,
        1 => MapCorner::BottomRight,
        2 => MapCorner::BottomLeft,
        _ => MapCorner::TopLeft,
    }
}
fn map_origin(n: u8, size: f32, width: f32, height: f32) -> (f32, f32) {
    let x = if n >= 2 { 2.0 } else { width - 2.0 - size };
    let y = if n == 0 || n == 3 {
        2.0
    } else {
        height - 2.0 - size
    };
    (x, y)
}
fn marker_shapes(
    radar: &RadarData,
    p: &Player,
    corner: u8,
    size: f32,
    width: f32,
    height: f32,
) -> Vec<Shape> {
    if radar.dimension != p.dimension || (corner == 0 && radar.effects) {
        return vec![];
    }
    let (x, y) = map_origin(corner, size, width, height);
    let mut shapes = Vec::new();
    let scale = size / 128.0; // map texture 128px, 1 block per map pixel, north up
    for (mx, mz, hostile) in &radar.markers {
        let dx = (*mx - p.x) as f32 * scale;
        let dz = (*mz - p.z) as f32 * scale;
        if dx.abs() > size / 2.0 - 3.0 || dz.abs() > size / 2.0 - 3.0 {
            continue;
        }
        shapes.push(Shape::Fill(FillShape {
            rect: Rect {
                x: x + size / 2.0 + dx - 1.5,
                y: y + size / 2.0 + dz - 1.5,
                w: 3.0,
                h: 3.0,
            },
            color: if *hostile { 0xffff5555 } else { 0xff55ff99 },
        }));
    }
    shapes
}

/// A session-tagged bounded in-process visual snapshot; no disk bridge.
fn normalized_xray_name(name: &str) -> Option<String> {
    let name = if name.contains(':') {
        name.to_owned()
    } else {
        format!("minecraft:{name}")
    };
    if name.len() > 48
        || name.starts_with(':')
        || name.ends_with(':')
        || name.matches(':').count() != 1
        || !name
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || b"_.:/-".contains(&c))
    {
        return None;
    }
    Some(name)
}
fn parse_xray_rules(text: &str) -> Option<Vec<(String, bool)>> {
    if text.is_empty() {
        return Some(Vec::new());
    }
    let mut result = Vec::new();
    for rule in text.split(',') {
        let (name, value) = rule.split_once('=')?;
        let name = normalized_xray_name(name)?;
        let show = match value {
            "1" => true,
            "0" => false,
            _ => return None,
        };
        if result.len() >= 16 || result.iter().any(|(n, _)| n == &name) {
            return None;
        }
        result.push((name, show));
    }
    result.sort_by(|a, b| a.0.cmp(&b.0));
    Some(result)
}
fn encode_xray_rules(rules: &[(String, bool)]) -> String {
    rules
        .iter()
        .map(|(n, v)| format!("{n}={}", u8::from(*v)))
        .collect::<Vec<_>>()
        .join(",")
}

fn visual_payload(
    session: &str,
    arc: bool,
    ore_esp: bool,
    mob_esp: bool,
    glow: bool,
    xray: bool,
    aim_walls: bool,
    tracers: bool,
    xray_mask: u32,
    xray_rules: &[(String, bool)],
) -> String {
    format!(
        "v4|{}|{}|{}|{}|{}|{}|{}|{}|{}|{}",
        session,
        u8::from(arc),
        u8::from(ore_esp),
        u8::from(mob_esp),
        u8::from(glow),
        u8::from(xray),
        u8::from(aim_walls),
        u8::from(tracers),
        xray_mask & XRAY_ALL,
        encode_xray_rules(xray_rules)
    )
}

struct ImpactPreview {
    settings: Settings,
    page: Page,
    own_screen: bool,
    last_panel: Vec<String>,
    point: Option<Point>,
    death_point: Option<Point>,
    last_health: Option<f32>,
    last_position: Option<(u64, f64, f64, String)>,
    speed: f64,
    zoom_held: bool,
    zoom_factor: f32,
    bridge: Option<BridgeStatus>,
    bridge_age: f32,
    bridge_poll: f32,
    bridge_visible: Option<(bool, bool, bool, bool)>,
    radar_data: Option<RadarData>,
    native: Option<NativeStatus>,
    had_shapes: bool,
}

impl ImpactPreview {
    fn widgets(&self) -> Vec<Widget> {
        let s = &self.settings;
        let toggle = |id, label: &str, value| button(id, format!("{label} {}", on_off(value)));
        let group = |id, text: &str| {
            Widget::Text(TextWidget {
                id,
                text: text.into(),
                color: 0x66CCFF,
            })
        };
        let short_corner = ["TR", "BR", "BL", "TL"][usize::from(s.map_corner.min(3))];
        vec![
            group(1000, "COMBAT"),
            button(ID_ATTACK, self.bridge_label("Attack", |b| b.attack)),
            button(ID_AIM, self.native_label("Aim", |n| n.aim)),
            button(ID_BOW, self.native_label("BowAim", |n| n.bow)),
            button(
                ID_PLAYER_BOW,
                self.native_label("Bow Players", |n| n.player_bow),
            ),
            button(
                ID_PLAYER_ATTACK,
                self.native_label("Attack Players", |n| n.player_attack),
            ),
            button(ID_CRITS, self.native_label("AutoCrit", |n| n.crits)),
            button(ID_FRIENDS, "Свои".into()),
            button(ID_CLEAR_FRIENDS, "Очистить список".into()),
            button(ID_LIST_FRIENDS, "Список своих".into()),
            toggle(ID_AIM_WALLS, "AimWalls", s.aim_walls),
            button(
                ID_RANGE,
                self.native
                    .as_ref()
                    .map_or("Range ...".into(), |n| format!("Range {:.1}", n.range)),
            ),
            button(
                ID_DELAY,
                self.native
                    .as_ref()
                    .map_or("Delay ...".into(), |n| format!("Delay {:.1}s", n.delay)),
            ),
            button(
                ID_TARGETS,
                self.native.as_ref().map_or("Targets ...".into(), |n| {
                    format!("Target {}", if n.all_mobs { "all" } else { "hostile" })
                }),
            ),
            button(
                ID_AIM_RADIUS,
                self.native
                    .as_ref()
                    .map_or("AimR ...".into(), |n| format!("AimR {:.0}", n.aim_radius)),
            ),
            group(1001, "WORLD"),
            button(ID_ORE, self.bridge_label("Find", |b| b.ore)),
            button(
                ID_ORE_TYPE,
                self.native
                    .as_ref()
                    .map_or("Ore ...".into(), |n| format!("Ore {}", n.ore)),
            ),
            button(
                ID_ORE_RADIUS,
                self.native
                    .as_ref()
                    .map_or("OreR ...".into(), |n| format!("OreR {}", n.radius)),
            ),
            button(ID_MINE, self.native_label("Mine", |n| n.mine)),
            button(ID_WALK, self.bridge_label("Walk", |b| b.walk)),
            button(ID_RADAR, self.bridge_label("Radar", |b| b.radar)),
            button(ID_BRIDGE_SELECT, "Bridge: select".into()),
            button(ID_STOP_ALL, "STOP ALL".into()),
            group(1002, "VISUAL"),
            toggle(ID_ARC, "Arc", s.arc),
            toggle(ID_ORE_ESP, "OreESP", s.ore_esp),
            toggle(ID_MOB_ESP, "MobESP", s.mob_esp),
            toggle(ID_ENTITY_GLOW, "Glow", s.entity_glow),
            toggle(ID_XRAY, "XRay", s.xray),
            toggle(ID_TRACERS, "Tracers", s.tracers),
            toggle(ID_MAP, "Minimap", s.map),
            toggle(ID_ZOOM, "Zoom", s.zoom),
            group(1003, "HUD"),
            toggle(ID_COORDS, "Coords", s.coords),
            toggle(ID_FACING, "Facing", s.facing),
            toggle(ID_CLOCK, "Clock", s.clock),
            toggle(ID_HEALTH, "Health", s.health),
            toggle(ID_FPS, "FPS", s.fps),
            toggle(ID_CPS, "CPS", s.cps),
            toggle(ID_SPEED, "Speed", s.speed),
            toggle(ID_DEATH, "Death", s.death),
            group(1004, "OPTIONS"),
            toggle(ID_WAYPOINT, "Waypoint", s.waypoint),
            button(ID_SAVE_POINT, "Save point".into()),
            button(ID_GO_POINT, "Go point".into()),
            button(ID_CLEAR_POINT, "Clear point".into()),
            button(ID_CLEAR_DEATH, "Clear death".into()),
            button(ID_MAP_CORNER, format!("Map {short_corner}")),
            button(ID_MAP_SIZE, format!("Map {}", s.map_size)),
            button(ID_MENU_BIND, "Menu key".into()),
            button(
                ID_XRAY_BASE,
                format!("X Diamond {}", on_off(s.xray_mask & (1 << 0) != 0)),
            ),
            button(
                ID_XRAY_BASE + 1,
                format!("X Iron {}", on_off(s.xray_mask & (1 << 1) != 0)),
            ),
            button(
                ID_XRAY_BASE + 2,
                format!("X Gold {}", on_off(s.xray_mask & (1 << 2) != 0)),
            ),
            button(
                ID_XRAY_BASE + 3,
                format!("X Redstone {}", on_off(s.xray_mask & (1 << 3) != 0)),
            ),
            button(
                ID_XRAY_BASE + 4,
                format!("X Emerald {}", on_off(s.xray_mask & (1 << 4) != 0)),
            ),
            button(
                ID_XRAY_BASE + 5,
                format!("X Coal {}", on_off(s.xray_mask & (1 << 5) != 0)),
            ),
            button(
                ID_XRAY_BASE + 6,
                format!("X Copper {}", on_off(s.xray_mask & (1 << 6) != 0)),
            ),
            button(
                ID_XRAY_BASE + 7,
                format!("X Lapis {}", on_off(s.xray_mask & (1 << 7) != 0)),
            ),
            button(
                ID_XRAY_BASE + 8,
                format!("X Quartz {}", on_off(s.xray_mask & (1 << 8) != 0)),
            ),
            button(
                ID_XRAY_BASE + 9,
                format!("X Debris {}", on_off(s.xray_mask & (1 << 9) != 0)),
            ),
            button(
                ID_XRAY_BASE + 10,
                format!("X Water {}", on_off(s.xray_mask & (1 << 10) != 0)),
            ),
            button(
                ID_XRAY_BASE + 11,
                format!("X Lava {}", on_off(s.xray_mask & (1 << 11) != 0)),
            ),
            button(
                ID_XRAY_BASE + 12,
                format!("X Other {}", on_off(s.xray_mask & (1 << 12) != 0)),
            ),
        ]
    }
    fn set_xray_bit(&mut self, bit: usize, value: Option<bool>) -> Result<(), String> {
        if bit >= 13 {
            return Err("Invalid X-Ray category".into());
        }
        let flag = 1u32 << bit;
        if value.unwrap_or(self.settings.xray_mask & flag == 0) {
            self.settings.xray_mask |= flag;
        } else {
            self.settings.xray_mask &= !flag;
        }
        storage::set("xray_mask", self.settings.xray_mask.to_string().as_bytes())
            .map_err(|e| format!("Cannot save X-Ray filter: {e}"))?;
        Ok(())
    }
    fn active_bridge(&self) -> Option<&BridgeStatus> {
        self.bridge
            .as_ref()
            .filter(|b| b.connected && self.bridge_age < BRIDGE_STALE_SECONDS)
    }
    fn bridge_label(&self, label: &str, value: impl FnOnce(&BridgeStatus) -> bool) -> String {
        match self.active_bridge() {
            Some(b) => format!("{label}: {}", on_off(value(b))),
            None => format!("{label}: NATIVE OFFLINE"),
        }
    }
    fn native_label(&self, label: &str, value: impl FnOnce(&NativeStatus) -> bool) -> String {
        match (self.active_bridge(), self.native.as_ref()) {
            (Some(_), Some(n)) => format!("{label}: {}", on_off(value(n))),
            _ => format!("{label}: NATIVE OFFLINE"),
        }
    }
    fn native_request(&self, command: &str, args: &[&str]) -> Result<(), String> {
        let b = self
            .active_bridge()
            .ok_or("Start the modified Impact Horizon Native client")?;
        if command.is_empty()
            || !command.bytes().all(|c| c.is_ascii_lowercase())
            || args.len() > 3
            || args.iter().any(|v| {
                v.len() > 40
                    || v.is_empty()
                    || !v
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || b".-_".contains(&c))
            })
        {
            return Err("Invalid native command".into());
        }
        let seq = stored(NATIVE_REQUEST)
            .and_then(|s| {
                let a: Vec<_> = s.split('|').collect();
                if a.len() >= 4 && a[0] == "v1" && a[1] == b.session {
                    a[2].parse::<u64>().ok()
                } else {
                    None
                }
            })
            .unwrap_or(0)
            .saturating_add(1);
        let mut text = format!("v1|{}|{}|{}", b.session, seq, command);
        for a in args {
            text.push('|');
            text.push_str(a);
        }
        storage::set(NATIVE_REQUEST, text.as_bytes())
            .map_err(|e| format!("Cannot send native request: {e}"))
    }
    /// Emit one bounded in-process flag request.
    fn bridge_request(&self, name: &str, value: Option<bool>) -> Result<bool, String> {
        let mut b = self
            .active_bridge()
            .cloned()
            .ok_or("Start the modified Impact Horizon Native client")?;
        let now = match name {
            "ore" => {
                b.ore = value.unwrap_or(!b.ore);
                b.ore
            }
            "attack" => {
                b.attack = value.unwrap_or(!b.attack);
                b.attack
            }
            "walk" => {
                b.walk = value.unwrap_or(!b.walk);
                b.walk
            }
            "radar" => {
                b.radar = value.unwrap_or(!b.radar);
                b.radar
            }
            "stop" => {
                b.ore = false;
                b.attack = false;
                b.walk = false;
                b.radar = false;
                false
            }
            _ => return Err("Unknown bridge module".into()),
        };
        // Read the last sequence so a mod reload within the same helper session
        // cannot accidentally repeat a request and be silently ignored.
        let seq = stored(BRIDGE_REQUEST)
            .and_then(|s| {
                let a: Vec<&str> = s.trim().split('|').collect();
                if a.len() == 5 && a[0] == "v1" && a[1] == b.session {
                    a[2].parse::<u64>().ok()
                } else {
                    None
                }
            })
            .unwrap_or(0)
            .saturating_add(1);
        storage::set(BRIDGE_REQUEST, b.request(seq, name, now).as_bytes())
            .map_err(|e| format!("Cannot send module request: {e}"))?;
        Ok(now)
    }
    fn update_bridge(&mut self, seconds: f32) {
        let elapsed = if seconds.is_finite() {
            seconds.clamp(0.0, 1.0)
        } else {
            0.0
        };
        self.bridge_age += elapsed;
        self.bridge_poll += elapsed;
        if self.bridge_poll < 0.25 {
            return;
        }
        self.bridge_poll = 0.0;
        let previous = self.bridge_visible;
        let previous_native = self.native.clone();
        let read = stored(BRIDGE_STATUS).and_then(|s| BridgeStatus::decode(&s));
        if let Some(next) = read {
            let new_beat = self
                .bridge
                .as_ref()
                .is_none_or(|old| old.session != next.session || old.heartbeat != next.heartbeat);
            if new_beat {
                self.bridge_age = 0.0;
            }
            self.bridge = Some(next);
        }
        let current = self
            .active_bridge()
            .map(|b| (b.ore, b.attack, b.walk, b.radar));
        self.bridge_visible = current;
        self.radar_data = stored(BRIDGE_RADAR).and_then(|s| RadarData::decode(&s));
        self.native = stored(NATIVE_STATUS).and_then(|s| NativeStatus::decode(&s));
        if let Some(b) = self.active_bridge() {
            let visual = visual_payload(
                &b.session,
                self.settings.arc,
                self.settings.ore_esp,
                self.settings.mob_esp,
                self.settings.entity_glow,
                self.settings.xray,
                self.settings.aim_walls,
                self.settings.tracers,
                self.settings.xray_mask,
                &self.settings.xray_rules,
            );
            // The host grants this key only to the authenticated adjacent mod.
            let _ = storage::set(NATIVE_VISUAL, visual.as_bytes());
        }
        if current != previous || self.native != previous_native {
            self.refresh_page();
        }
    }
    fn refresh_page(&self) {
        let widgets = self.widgets();
        if self.own_screen {
            ui::open_screen(&Screen {
                title: "Impact Horizon Native".into(),
                widgets,
            });
        } else {
            ui::set_settings(&widgets);
        }
    }
    fn open_menu(&mut self) {
        self.end_zoom();
        self.page = Page::Home;
        self.own_screen = true;
        self.refresh_page();
    }
    fn end_zoom(&mut self) {
        if self.zoom_held {
            self.zoom_held = false;
            input::claim_wheel(false);
            render::set_zoom(Zoom {
                factor: 1.0,
                smooth: true,
                turn: 1.0,
            });
        }
    }
    fn show_zoom(&self) {
        let factor = if self.zoom_held {
            self.zoom_factor
        } else {
            1.0
        };
        render::set_zoom(Zoom {
            factor,
            smooth: true,
            turn: (1.0 / factor).max(0.05),
        });
    }
    fn apply_map(&self) {
        let m = self.settings.map.then_some(Minimap {
            corner: map_corner(self.settings.map_corner),
            size: f32::from(self.settings.map_size),
            blocks_per_pixel: 1.0,
            rotate: false,
            round: false,
        });
        render::set_minimap(m);
    }
    fn next_map_corner(&mut self) -> Result<(), String> {
        let n = (self.settings.map_corner + 1) % 4;
        storage::set("map_corner", n.to_string().as_bytes())?;
        self.settings.map_corner = n;
        self.apply_map();
        Ok(())
    }
    fn next_map_size(&mut self) -> Result<(), String> {
        let size = match self.settings.map_size {
            96 => 192,
            192 => 256,
            _ => 96,
        };
        storage::set("map_size", size.to_string().as_bytes())?;
        self.settings.map_size = size;
        self.apply_map();
        Ok(())
    }
    fn draw_shapes(&mut self, p: &Player) {
        let (width, height) = render::gui_size();
        let mut shapes = Vec::new();
        if self.settings.map
            && let Some(b) = self.active_bridge()
            && b.radar
            && let Some(r) = &self.radar_data
            && r.session == b.session
        {
            shapes.extend(marker_shapes(
                r,
                p,
                self.settings.map_corner,
                f32::from(self.settings.map_size),
                width,
                height,
            ));
        }
        if !shapes.is_empty() || self.had_shapes {
            render::set_hud(&shapes);
            self.had_shapes = !shapes.is_empty();
        }
    }
    fn set_flag(&mut self, name: &str, value: Option<bool>) -> Result<bool, String> {
        let flag = self
            .settings
            .flag_mut(name)
            .ok_or_else(|| format!("Unknown module: {name}"))?;
        let previous = *flag;
        let now = value.unwrap_or(!previous);
        // Do not report a permanent change if the sandbox cannot save it.
        if name != "xray" {
            save_flag(name, now)?;
        }
        *flag = now;
        if name == "map" {
            self.apply_map();
        }
        if name == "zoom" && !now {
            self.end_zoom();
        }
        Ok(now)
    }
    fn save_current_point(&mut self) -> Result<String, String> {
        let p = game::get_player().ok_or("Open a world first")?;
        let point = Point::from_player(&p);
        let short = point.short();
        save_point("point", &point)?;
        self.point = Some(point);
        Ok(short)
    }
    fn tick_hud(&mut self, tick: u64, p: &Player) {
        if self.settings.death && self.last_health.is_some_and(|h| h > 0.0) && p.health <= 0.0 {
            let death = Point::from_player(p);
            match save_point("death_point", &death) {
                Ok(()) => chat::show(&format!("Death coordinates recorded: {}", death.short())),
                Err(e) => chat::show(&format!("{e}; death position is temporary")),
            }
            self.death_point = Some(death);
        }
        self.last_health = Some(p.health);
        if let Some((last_tick, x, z, dim)) = &self.last_position {
            let dt = tick.saturating_sub(*last_tick);
            if dt > 0 && dt <= 100 && *dim == p.dimension {
                self.speed = ((p.x - x).powi(2) + (p.z - z).powi(2)).sqrt() * 20.0 / dt as f64;
            } else {
                self.speed = 0.0;
            }
        }
        self.last_position = Some((tick, p.x, p.z, p.dimension.clone()));
        let s = &self.settings;
        let mut lines = Vec::<String>::new();
        if s.coords {
            lines.push(format!("XYZ {:.1} / {:.1} / {:.1}", p.x, p.y, p.z));
        }
        if s.facing {
            lines.push(format!("Facing {}", facing(p.yaw)));
        }
        if s.clock {
            if let Some(c) = game::get_clock() {
                lines.push(time_of_day(&c));
            }
        }
        if s.health {
            lines.push(format!("Health {:.0}  Food {}/20", p.health, p.food));
        }
        if s.fps {
            lines.push(format!("FPS {}", render::frame_stats().fps));
        }
        if s.cps {
            let (l, r) = input::clicks_per_second();
            lines.push(format!("CPS Attack {l} / Use {r}"));
        }
        if s.speed {
            lines.push(format!("Speed {:.1} blocks/s", self.speed));
        }
        if s.waypoint {
            if let Some(point) = &self.point {
                let distance = point
                    .distance(p)
                    .map_or_else(|| "other dimension".into(), |d| format!("{d:.0} blocks"));
                lines.push(format!("Waypoint {} ({distance})", point.short()));
            }
        }
        if s.death {
            if let Some(point) = &self.death_point {
                let distance = point
                    .distance(p)
                    .map_or_else(|| "other dimension".into(), |d| format!("{d:.0} blocks"));
                lines.push(format!("Death {} ({distance})", point.short()));
            }
        }
        if let Some(b) = self.active_bridge() {
            if b.ore {
                if b.ore_kind != "none" && b.dimension == p.dimension {
                    let [x, y, z] = b.ore_pos;
                    let dist = ((p.x - (x as f64 + 0.5)).powi(2)
                        + (p.y - (y as f64 + 0.5)).powi(2)
                        + (p.z - (z as f64 + 0.5)).powi(2))
                    .sqrt();
                    lines.push(format!("Ore {}: {x} / {y} / {z} ({dist:.0}m)", b.ore_kind));
                } else {
                    lines.push("Ore Finder: searching / no result".into());
                }
            }
            if b.radar {
                if b.radar_kind != "none" && b.dimension == p.dimension {
                    let [x, y, z] = b.radar_pos;
                    lines.push(format!(
                        "Mob radar: {} | {} {x}/{y}/{z}",
                        b.radar_count, b.radar_kind
                    ));
                } else {
                    lines.push(format!("Mob radar: {}", b.radar_count));
                }
            }
            if b.attack {
                lines.push("AutoAttack armed (configured in game)".into());
            }
            if b.walk {
                lines.push("AutoWalk armed".into());
            }
        }
        if !lines.is_empty() {
            lines.insert(0, "Impact Horizon Native".into());
        }
        if lines != self.last_panel {
            if lines.is_empty() {
                hud::clear();
            } else {
                let panel: Vec<Line> = lines
                    .iter()
                    .enumerate()
                    .map(|(i, s)| Line {
                        text: s.clone(),
                        color: if i == 0 { 0x64E1BC } else { 0xFFFFFF },
                    })
                    .collect();
                hud::set_panel(HudCorner::TopLeft, &panel);
            }
            self.last_panel = lines;
        }
    }
}

impl Mod for ImpactPreview {
    fn init() -> Result<(Self, Registration), String> {
        input::register_key(MENU_KEY, "Impact Native menu", "key.keyboard.f8")?;
        input::register_key(ZOOM_KEY, "Impact Native zoom", "key.keyboard.z")?;
        for (name, label) in [
            ("ih_attack", "Attack"),
            ("ih_aim", "Aim"),
            ("ih_bow", "Bow Aim"),
            ("ih_walls", "Aim through walls"),
            ("ih_ore", "Ore Finder"),
            ("ih_mine", "AutoMine"),
            ("ih_walk", "AutoWalk"),
            ("ih_radar", "Radar"),
            ("ih_arc", "Trajectory"),
            ("ih_oreesp", "Ore ESP"),
            ("ih_mobesp", "Mob ESP"),
            ("ih_glow", "Entity Glow"),
            ("ih_xray", "X-Ray"),
            ("ih_stop", "STOP ALL"),
            ("ih_pattack", "Attack Players"),
            ("ih_pbow", "Bow Players"),
            ("ih_crits", "Auto criticals"),
            ("ih_friends", "Add friend"),
            ("ih_bridge", "Bridge builder"),
            ("ih_tracers", "Enemy tracers"),
        ] {
            input::register_key(name, label, "key.keyboard.unknown")?;
        }
        let me = Self {
            settings: Settings::load(),
            page: Page::Home,
            own_screen: false,
            last_panel: Vec::new(),
            point: read_point("point"),
            death_point: read_point("death_point"),
            last_health: None,
            last_position: None,
            speed: 0.0,
            zoom_held: false,
            zoom_factor: 4.0,
            bridge: None,
            bridge_age: 999.0,
            bridge_poll: 1.0,
            bridge_visible: None,
            radar_data: None,
            native: None,
            had_shapes: false,
        };
        me.apply_map();
        me.refresh_page();
        Ok((me, Registration::new().command("ih").on_tick().on_frame()))
    }
    fn tick(&mut self, tick: u64) {
        if let Some(p) = game::get_player() {
            self.tick_hud(tick, &p);
            self.draw_shapes(&p);
        } else {
            // X-Ray starts disabled when entering a new world/session.
            self.settings.xray = false;
            if self.had_shapes {
                render::set_hud(&[]);
                self.had_shapes = false;
            }
            if !self.last_panel.is_empty() {
                hud::clear();
                self.last_panel.clear();
            }
            self.last_health = None;
            self.last_position = None;
            self.speed = 0.0;
        }
    }
    fn frame(&mut self, seconds: f32) {
        self.update_bridge(seconds);
        for event in input::poll() {
            match event {
                Event::KeyDown(k) if k == MENU_KEY => self.open_menu(),
                Event::KeyDown(k) if k.starts_with("ih_") => {
                    let id = match k.as_str() {
                        "ih_attack" => ID_ATTACK,
                        "ih_aim" => ID_AIM,
                        "ih_bow" => ID_BOW,
                        "ih_walls" => ID_AIM_WALLS,
                        "ih_ore" => ID_ORE,
                        "ih_mine" => ID_MINE,
                        "ih_walk" => ID_WALK,
                        "ih_radar" => ID_RADAR,
                        "ih_arc" => ID_ARC,
                        "ih_oreesp" => ID_ORE_ESP,
                        "ih_mobesp" => ID_MOB_ESP,
                        "ih_glow" => ID_ENTITY_GLOW,
                        "ih_xray" => ID_XRAY,
                        "ih_stop" => ID_STOP_ALL,
                        "ih_pattack" => ID_PLAYER_ATTACK,
                        "ih_pbow" => ID_PLAYER_BOW,
                        "ih_crits" => ID_CRITS,
                        "ih_friends" => ID_FRIENDS,
                        "ih_bridge" => ID_BRIDGE_SELECT,
                        "ih_tracers" => ID_TRACERS,
                        _ => continue,
                    };
                    self.ui_select(id, 0);
                }
                Event::KeyDown(k) if k == ZOOM_KEY && self.settings.zoom => {
                    self.zoom_held = true;
                    self.zoom_factor = 4.0;
                    input::claim_wheel(true);
                    self.show_zoom();
                }
                Event::KeyUp(k) if k == ZOOM_KEY => self.end_zoom(),
                Event::Wheel(steps) if self.zoom_held => {
                    self.zoom_factor = (self.zoom_factor * 1.25_f32.powf(steps)).clamp(1.25, 10.0);
                    self.show_zoom();
                }
                _ => {}
            }
        }
    }
    fn command(&mut self, _name: &str, args: &str) -> Result<Option<String>, String> {
        let mut words = args.split_whitespace();
        let sub = words.next().unwrap_or("menu").to_ascii_lowercase();
        match sub.as_str() {
            "menu" => {
                if game::get_player().is_none() { return Err("Open a world first".into()); }
                self.open_menu(); Ok(Some("Impact Native menu opened".into()))
            }
            "help" => Ok(Some("/ih menu | friends select/list/clear | bridge | pattack/pbow/crits/tracers on|off | xray on|off | xrayfilter CATEGORY on|off | xrayblock ID on|off|default | goto X Y Z | waypoint save/go | stop".into())),
            "ore" | "attack" | "walk" | "radar" => {
                let value = match words.next() { None => None, Some("on") => Some(true), Some("off") => Some(false), _ => return Err("Use on or off".into()) };
                let now = self.bridge_request(&sub, value)?;
                Ok(Some(format!("Request sent: {sub} {}", on_off(now))))
            }
            "stop" => {
                self.bridge_request("stop", Some(false))?;
                for name in ["arc", "espore", "espmobs", "glow", "xray", "tracers"] { self.set_flag(name, Some(false))?; }
                Ok(Some("Stop request sent; 3D overlays disabled".into()))
            }
            "goto" => {
                let xyz: Vec<_> = words.collect();
                if xyz.len()!=3 {return Err("Use /ih goto X Y Z (max 2048 blocks)".into());}
                for v in &xyz { if !v.parse::<f64>().ok().is_some_and(f64::is_finite) {return Err("Coordinates must be finite numbers".into());} }
                self.native_request("goto",&xyz)?; Ok(Some("Navigation request sent; close chat".into()))
            }
            "friends" => {
                let arg=words.next().unwrap_or("select");
                if arg!="select" && arg!="list" && arg!="clear" {return Err("Use /ih friends select, list or clear".into());}
                self.native_request("friends", &[arg])?;
                Ok(Some("Friends request sent".into()))
            }
            "bridge" => { self.native_request("bridgeselect", &[])?; Ok(Some("Select two blocks with right click".into())) }
            "xrayblock" => {
                let name=words.next().ok_or("Use /ih xrayblock stone on|off|default or clear")?;
                if name=="clear" {
                    if words.next().is_some() {return Err("Use /ih xrayblock clear".into());}
                    self.settings.xray_rules.clear();
                } else {
                    let name=normalized_xray_name(name).ok_or("Invalid block id")?;
                    let mode=words.next().ok_or("Use on, off or default")?;
                    if words.next().is_some() {return Err("Use one block and on/off/default".into());}
                    let before=self.settings.xray_rules.clone();
                    self.settings.xray_rules.retain(|(n,_)|n!=&name);
                    match mode {
                        "on"|"off" => {
                            if self.settings.xray_rules.len()>=16 {self.settings.xray_rules=before;return Err("Maximum 16 block overrides".into());}
                            self.settings.xray_rules.push((name.clone(),mode=="on"));
                            self.settings.xray_rules.sort_by(|a,b|a.0.cmp(&b.0));
                        }
                        "default"=>{}, _=>{self.settings.xray_rules=before;return Err("Use on, off or default".into());}
                    }
                }
                storage::set("xray_rules",encode_xray_rules(&self.settings.xray_rules).as_bytes()).map_err(|e|format!("Cannot save X-Ray block rules: {e}"))?;
                Ok(Some("X-Ray block rules updated".into()))
            }
            "xrayfilter" => {
                let name=words.next().ok_or("Use /ih xrayfilter diamond on|off")?;
                let val=words.next().ok_or("Use on or off")?;
                if words.next().is_some() {return Err("Use one category and on/off".into());}
                let bit=XRAY_NAMES.iter().position(|(key,_)|*key==name).ok_or("Unknown X-Ray category")?;
                let value=match val {"on"=>true,"off"=>false,_=>return Err("Use on or off".into())};
                self.set_xray_bit(bit, Some(value))?;
                Ok(Some(format!("X-Ray {name}: {}",on_off(value))))
            }
            "range" | "delay" | "targets" | "oretype" | "oreradius" | "aimradius" | "aim" | "bow" | "mine" | "pattack" | "pbow" | "crits" => {
                let value=words.next().ok_or("Give a value; use /ih help")?;
                if words.next().is_some() {return Err("Only one value expected".into());}
                self.native_request(&sub,&[value])?;
                Ok(Some(format!("In-game request: {sub} {value}")))
            }
            "mapcorner" => { self.next_map_corner()?; Ok(Some(format!("Map position: {}", corner_name(self.settings.map_corner)))) }
            "mapsize" => { self.next_map_size()?; Ok(Some(format!("Map size: {}", self.settings.map_size))) }
            "waypoint" | "wp" => match words.next().unwrap_or("show") {
                "save" => Ok(Some(format!("Waypoint saved: {}", self.save_current_point()?))),
                "clear" => { self.point = None; storage::delete("point"); Ok(Some("Waypoint cleared".into())) }
                "go" => {
                    let p=self.point.as_ref().ok_or("Save a waypoint first")?;
                    let here=game::get_player().ok_or("Open a world first")?;
                    if p.dimension!=here.dimension {return Err("Waypoint is in another dimension".into());}
                    self.native_request("goto",&[&p.x.to_string(),&p.y.to_string(),&p.z.to_string()])?;
                    Ok(Some("Walking to saved waypoint after chat closes".into()))
                }
                "show" => Ok(Some(self.point.as_ref().map_or("No waypoint saved".into(), |p| format!("Waypoint: {} ({})", p.short(), p.dimension)))),
                _ => Err("Use /ih waypoint save, go, clear or show".into()),
            },
            "coords" | "facing" | "clock" | "health" | "fps" | "cps" | "speed" | "map" | "zoom" | "death" | "arc" | "espore" | "espmobs" | "glow" | "xray" | "aimwalls" | "tracers" => {
                let value = match words.next() {
                    None => None, Some("on") => Some(true), Some("off") => Some(false),
                    _ => return Err("Use on or off".into()),
                };
                let now = self.set_flag(&sub, value)?;
                Ok(Some(format!("{sub}: {}", on_off(now))))
            }
            _ => Err("Unknown option. Use /ih help".into()),
        }
    }
    fn ui_select(&mut self, id: u32, _row: u32) {
        let name = match id {
            ID_COORDS => Some("coords"),
            ID_FACING => Some("facing"),
            ID_CLOCK => Some("clock"),
            ID_HEALTH => Some("health"),
            ID_FPS => Some("fps"),
            ID_CPS => Some("cps"),
            ID_SPEED => Some("speed"),
            ID_MAP => Some("map"),
            ID_ARC => Some("arc"),
            ID_ORE_ESP => Some("espore"),
            ID_MOB_ESP => Some("espmobs"),
            ID_ENTITY_GLOW => Some("glow"),
            ID_XRAY => Some("xray"),
            ID_AIM_WALLS => Some("aimwalls"),
            ID_TRACERS => Some("tracers"),
            ID_ZOOM => Some("zoom"),
            ID_WAYPOINT => Some("waypoint"),
            ID_DEATH => Some("death"),
            _ => None,
        };
        if let Some(name) = name {
            if let Err(e) = self.set_flag(name, None) {
                chat::show(&e);
            }
        } else {
            match id {
                ID_BACK => {
                    self.page = match self.page {
                        Page::PointsMore => Page::Waypoints,
                        Page::Display | Page::Stats => Page::Hud,
                        Page::ModulesMore => Page::Modules,
                        Page::ModulesAdvanced => Page::ModulesMore,
                        Page::ModulesAim => Page::ModulesAdvanced,
                        Page::CombatSettings => Page::ModulesAim,
                        Page::MapOptions => Page::Tools,
                        Page::MapMore => Page::MapOptions,
                        Page::Visuals => Page::MapMore,
                        Page::SearchSettings => Page::Visuals,
                        _ => Page::Home,
                    }
                }
                ID_HUD => self.page = Page::Hud,
                ID_MODULES => self.page = Page::Modules,
                ID_MODULES_MORE => self.page = Page::ModulesMore,
                ID_MODULES_ADVANCED => self.page = Page::ModulesAdvanced,
                ID_MODULES_AIM => self.page = Page::ModulesAim,
                ID_COMBAT_SETTINGS => self.page = Page::CombatSettings,
                ID_SEARCH_SETTINGS => self.page = Page::SearchSettings,
                ID_ORE_RADIUS | ID_AIM_RADIUS => {
                    if let Some(n) = self.native.as_ref() {
                        let (kind, value) = if id == ID_ORE_RADIUS {
                            (
                                "oreradius",
                                if n.radius < 16 {
                                    "16"
                                } else if n.radius < 32 {
                                    "32"
                                } else if n.radius < 64 {
                                    "64"
                                } else if n.radius < 96 {
                                    "96"
                                } else if n.radius < 128 {
                                    "128"
                                } else {
                                    "8"
                                },
                            )
                        } else {
                            (
                                "aimradius",
                                if n.aim_radius < 16.0 {
                                    "16"
                                } else if n.aim_radius < 32.0 {
                                    "32"
                                } else if n.aim_radius < 48.0 {
                                    "48"
                                } else if n.aim_radius < 64.0 {
                                    "64"
                                } else if n.aim_radius < 96.0 {
                                    "96"
                                } else if n.aim_radius < 128.0 {
                                    "128"
                                } else {
                                    "8"
                                },
                            )
                        };
                        if let Err(e) = self.native_request(kind, &[value]) {
                            chat::show(&e);
                        }
                    } else {
                        chat::show("Native backend offline");
                    }
                }
                ID_PLAYER_ATTACK | ID_PLAYER_BOW | ID_CRITS => {
                    let mode = match id {
                        ID_PLAYER_ATTACK => "pattack",
                        ID_PLAYER_BOW => "pbow",
                        _ => "crits",
                    };
                    let is_on = self.native.as_ref().is_some_and(|n| match id {
                        ID_PLAYER_ATTACK => n.player_attack,
                        ID_PLAYER_BOW => n.player_bow,
                        _ => n.crits,
                    });
                    if let Err(e) = self.native_request(mode, &[if is_on { "off" } else { "on" }]) {
                        chat::show(&e);
                    }
                }
                ID_FRIENDS | ID_LIST_FRIENDS | ID_BRIDGE_SELECT => {
                    let result = match id {
                        ID_FRIENDS => self.native_request("friends", &["select"]),
                        ID_LIST_FRIENDS => self.native_request("friends", &["list"]),
                        _ => self.native_request("bridgeselect", &[]),
                    };
                    match result {
                        Ok(()) => {
                            if self.own_screen {
                                self.own_screen = false;
                                ui::close_screen();
                            }
                        }
                        Err(e) => chat::show(&e),
                    }
                }
                ID_CLEAR_FRIENDS => {
                    if let Err(e) = self.native_request("friends", &["clear"]) {
                        chat::show(&e);
                    }
                }
                id if (ID_XRAY_BASE..ID_XRAY_BASE + 13).contains(&id) => {
                    if let Err(e) = self.set_xray_bit((id - ID_XRAY_BASE) as usize, None) {
                        chat::show(&e);
                    }
                }
                ID_AIM | ID_BOW | ID_MINE => {
                    let mode = match id {
                        ID_AIM => "aim",
                        ID_BOW => "bow",
                        _ => "mine",
                    };
                    let is_on = self.native.as_ref().is_some_and(|n| match id {
                        ID_AIM => n.aim,
                        ID_BOW => n.bow,
                        _ => n.mine,
                    });
                    if let Err(e) = self.native_request(mode, &[if is_on { "off" } else { "on" }]) {
                        chat::show(&e);
                    }
                }
                ID_ORE_TYPE => {
                    if let Some(n) = self.native.as_ref() {
                        let types = [
                            "diamond", "iron", "gold", "redstone", "emerald", "coal", "copper",
                            "lapis", "quartz", "debris",
                        ];
                        let i = types.iter().position(|v| *v == n.ore).unwrap_or(0);
                        if let Err(e) =
                            self.native_request("oretype", &[types[(i + 1) % types.len()]])
                        {
                            chat::show(&e);
                        }
                    }
                }
                ID_GO_POINT => {
                    if let Some(p) = self.point.as_ref() {
                        if game::get_player().is_some_and(|here| here.dimension == p.dimension) {
                            if let Err(e) = self.native_request(
                                "goto",
                                &[&p.x.to_string(), &p.y.to_string(), &p.z.to_string()],
                            ) {
                                chat::show(&e);
                            } else {
                                chat::show("Navigation queued; close the menu.");
                            }
                        } else {
                            chat::show("Waypoint is in another dimension.");
                        }
                    } else {
                        chat::show("Save a waypoint first.");
                    }
                }
                ID_MENU_BIND => {
                    chat::show("Right-click Menu key, then press the new key.");
                }
                ID_RANGE | ID_DELAY | ID_TARGETS => {
                    if let Some(n) = self.native.as_ref() {
                        let (kind, value) = match id {
                            ID_RANGE => (
                                "range",
                                if n.range < 2.0 {
                                    "2"
                                } else if n.range < 2.5 {
                                    "2.5"
                                } else if n.range < 3.0 {
                                    "3"
                                } else if n.range < 3.5 {
                                    "3.5"
                                } else if n.range < 4.0 {
                                    "4"
                                } else if n.range < 5.0 {
                                    "5"
                                } else if n.range < 6.0 {
                                    "6"
                                } else {
                                    "1.5"
                                },
                            ),
                            ID_DELAY => (
                                "delay",
                                if n.delay < 0.2 {
                                    "0.2"
                                } else if n.delay < 0.3 {
                                    "0.3"
                                } else if n.delay < 0.5 {
                                    "0.5"
                                } else if n.delay < 1.0 {
                                    "1"
                                } else if n.delay < 2.0 {
                                    "2"
                                } else {
                                    "0.1"
                                },
                            ),
                            _ => ("targets", if n.all_mobs { "hostile" } else { "all" }),
                        };
                        if let Err(e) = self.native_request(kind, &[value]) {
                            chat::show(&e);
                        }
                    } else {
                        chat::show("Native backend offline");
                    }
                }
                ID_ORE | ID_ATTACK | ID_RADAR | ID_WALK | ID_STOP_ALL => {
                    if id == ID_STOP_ALL {
                        for name in ["arc", "espore", "espmobs", "glow", "xray", "tracers"] {
                            if let Err(e) = self.set_flag(name, Some(false)) {
                                chat::show(&e);
                            }
                        }
                    }
                    let n = match id {
                        ID_ORE => "ore",
                        ID_ATTACK => "attack",
                        ID_RADAR => "radar",
                        ID_WALK => "walk",
                        _ => "stop",
                    };
                    match self.bridge_request(n, None) {
                        Ok(_) => chat::show("In-game request sent"),
                        Err(e) => chat::show(&e),
                    }
                }
                ID_DISPLAY => self.page = Page::Display,
                ID_STATS => self.page = Page::Stats,
                ID_TOOLS => self.page = Page::Tools,
                ID_MAP_OPTIONS => self.page = Page::MapOptions,
                ID_MAP_MORE => self.page = Page::MapMore,
                ID_VISUALS => self.page = Page::Visuals,
                ID_MAP_CORNER => {
                    if let Err(e) = self.next_map_corner() {
                        chat::show(&e);
                    }
                }
                ID_MAP_SIZE => {
                    if let Err(e) = self.next_map_size() {
                        chat::show(&e);
                    }
                }
                ID_WAYPOINTS => self.page = Page::Waypoints,
                ID_POINTS_MORE => self.page = Page::PointsMore,
                ID_SAVE_POINT => match self.save_current_point() {
                    Ok(pos) => chat::show(&format!("Waypoint saved: {pos}")),
                    Err(e) => chat::show(&e),
                },
                ID_CLEAR_POINT => {
                    self.point = None;
                    storage::delete("point");
                }
                ID_CLEAR_DEATH => {
                    self.death_point = None;
                    storage::delete("death_point");
                }
                _ => return,
            }
        }
        self.refresh_page();
    }
    fn ui_closed(&mut self) {
        self.own_screen = false;
        self.page = Page::Home;
    }
    fn ui_settings_opened(&mut self) {
        self.own_screen = false;
        self.page = Page::Home;
        self.refresh_page();
    }
}

horizon_mod_sdk::export_mod!(ImpactPreview);

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn directions_and_time() {
        assert_eq!(facing(0.0), "South (+Z)");
        assert_eq!(facing(90.0), "West (-X)");
        assert_eq!(facing(-180.0), "North (-Z)");
        assert_eq!(facing(-90.0), "East (+X)");
        assert_eq!(
            time_of_day(&Clock {
                game_time: 0,
                day_time: 0
            }),
            "Day 1, 06:00"
        );
        assert_eq!(
            time_of_day(&Clock {
                game_time: 0,
                day_time: 18000
            }),
            "Day 1, 00:00"
        );
    }
    #[test]
    fn point_storage_round_trip_and_validation() {
        let p = Point {
            x: -3.5,
            y: 70.0,
            z: 8.25,
            dimension: "minecraft:overworld".into(),
        };
        assert_eq!(p.short(), "-4 / 70 / 8");
        assert_eq!(Point::decode(&p.encode()), Some(p));
        assert!(Point::decode("NaN|0|1|minecraft:overworld").is_none());
        assert!(Point::decode("1|2|3|a|extra").is_none());
        assert!(Point::decode("1|2|3|").is_none());
    }
    #[test]
    fn pages_fit_standard_horizon_gui_even_with_key_rows() {
        let mut m = ImpactPreview {
            settings: Settings {
                coords: true,
                facing: true,
                clock: false,
                health: false,
                fps: false,
                cps: false,
                speed: false,
                map: false,
                map_corner: 0,
                map_size: 96,
                arc: false,
                ore_esp: false,
                mob_esp: false,
                entity_glow: false,
                xray: false,
                aim_walls: false,
                tracers: false,
                xray_mask: XRAY_ALL,
                xray_rules: Vec::new(),
                zoom: true,
                waypoint: true,
                death: false,
            },
            page: Page::Home,
            own_screen: false,
            last_panel: Vec::new(),
            point: None,
            death_point: None,
            last_health: None,
            last_position: None,
            speed: 0.0,
            zoom_held: false,
            zoom_factor: 4.0,
            bridge: None,
            bridge_age: 999.0,
            bridge_poll: 1.0,
            bridge_visible: None,
            radar_data: None,
            native: None,
            had_shapes: false,
        };
        let widgets = m.widgets();
        assert!(widgets.len() <= 96);
        let mut groups = 0;
        let mut count = 0;
        let mut ids = Vec::new();
        for w in widgets {
            match w {
                Widget::Text(_) => {
                    assert!(count <= 24);
                    groups += 1;
                    count = 0;
                }
                Widget::Button(b) => {
                    assert!(!ids.contains(&b.id));
                    ids.push(b.id);
                    count += 1;
                }
                _ => panic!("unexpected widget"),
            }
        }
        assert!(count <= 24);
        assert_eq!(groups, 5);
        assert!(
            ids.contains(&ID_XRAY) && ids.contains(&ID_ENTITY_GLOW) && ids.contains(&ID_AIM_WALLS)
        );
    }
    #[test]
    fn flags_reject_unknown_values() {
        assert!(parse_flag(Some("true"), false));
        assert!(!parse_flag(Some("false"), true));
        assert!(parse_flag(Some("garbage"), true));
    }
    #[test]
    fn radar_markers_are_bounded_and_session_tagged() {
        let text = format!(
            "v1|{}|minecraft:overworld|0|10.5,-4.2,1;11,4,0",
            "a".repeat(32)
        );
        let r = RadarData::decode(&text).unwrap();
        assert_eq!(r.markers.len(), 2);
        assert!(r.markers[0].2);
        assert!(RadarData::decode(&text.replace("10.5", "NaN")).is_none());
        assert!(RadarData::decode(&text.replace("|0|", "|2|")).is_none());
        assert_eq!(map_origin(0, 96.0, 640.0, 360.0), (542.0, 2.0));
        assert_eq!(map_origin(2, 96.0, 640.0, 360.0), (2.0, 262.0));
    }
    #[test]
    fn visual_snapshot_is_session_tagged_and_bounded() {
        assert_eq!(
            visual_payload(
                "0123456789abcdef0123456789abcdef",
                true,
                false,
                true,
                true,
                false,
                true,
                true,
                XRAY_ALL,
                &[]
            ),
            "v4|0123456789abcdef0123456789abcdef|1|0|1|1|0|1|1|8191|"
        );
    }
    #[test]
    fn xray_specific_rules_are_bounded_and_strict() {
        assert_eq!(
            normalized_xray_name("stone").as_deref(),
            Some("minecraft:stone")
        );
        assert!(normalized_xray_name("../evil=x").is_none());
        let rules = parse_xray_rules("minecraft:stone=1,minecraft:diamond_ore=0").unwrap();
        assert_eq!(rules.len(), 2);
        assert!(parse_xray_rules("minecraft:stone=1,minecraft:stone=0").is_none());
        assert!(parse_xray_rules("minecraft:stone=2").is_none());
        assert!(
            parse_xray_rules(
                &(0..17)
                    .map(|i| format!("minecraft:b{i}=1"))
                    .collect::<Vec<_>>()
                    .join(",")
            )
            .is_none()
        );
        assert_eq!(parse_xray_rules(&encode_xray_rules(&rules)), Some(rules));
    }
    #[test]
    fn native_status_has_bounded_validated_values() {
        let n = NativeStatus::decode("v1|1|0|1|3.0|1.0|hostile|diamond|12|0").unwrap();
        assert!(n.aim && !n.bow && n.mine && !n.all_mobs && !n.nav);
        assert_eq!(n.ore, "diamond");
        assert!(NativeStatus::decode("v1|1|0|1|NaN|1.0|hostile|diamond|12|0").is_none());
        assert!(NativeStatus::decode("v1|1|0|1|3.0|0.01|hostile|diamond|12|0").is_none());
        assert!(NativeStatus::decode("v1|1|0|1|3.0|1.0|hostile|diamond|129|0").is_none());
        assert!(NativeStatus::decode("v1|1|0|1|6.0|0.1|hostile|diamond|128|0|128").is_some());
        assert!(NativeStatus::decode("v1|1|0|1|6.0|0.1|hostile|diamond|128|0|129").is_none());
    }
    #[test]
    fn bridge_protocol_is_bounded_and_strict() {
        let ok = "v1|0123456789abcdef0123456789abcdef|9|1|1|0|1|1|diamond|minecraft:overworld|-3|70|8|2|zombie|10|71|4";
        let b = BridgeStatus::decode(ok).unwrap();
        assert!(b.connected && b.ore && b.walk && b.radar && !b.attack);
        assert_eq!(b.ore_pos, [-3, 70, 8]);
        assert_eq!(
            b.request(10, "ore", true),
            "v1|0123456789abcdef0123456789abcdef|10|ore|1"
        );
        assert!(BridgeStatus::decode(&ok.replace("|1|1|0|", "|2|1|0|")).is_none());
        assert!(BridgeStatus::decode(&ok.replace("minecraft:overworld", "bad|path")).is_none());
        assert!(BridgeStatus::decode(&format!("{ok}|trailing")).is_none());
        assert!(BridgeStatus::decode(&ok.replace("|2|zombie", "|65535|zombie")).is_none());
    }
}
