// Run against patched Horizon v0.2.92 horizon-mods with `--features runtime`.
// This checks the actual Wasmtime host and bundled .hmod, without any game,
// disk bridge, external panel or Agent Control.
use horizon_mods::{Mods, Output, Snapshot, Status, UiWidget, testing::default_player};
use std::{env, fs, path::PathBuf};

fn label(mods: &Mods, id: u32) -> String {
    mods.screen(0).unwrap().widgets.iter().find_map(|w| match w {
        UiWidget::Button(b) if b.id == id => Some(b.label.clone()), _ => None,
    }).unwrap_or_else(|| panic!("missing UI button {id}"))
}
fn main() {
    let original=PathBuf::from(env::args_os().nth(1).expect(".hmod path"));
    let root=env::temp_dir().join(format!("impact-native-memory-{}",std::process::id()));
    let _=fs::remove_dir_all(&root); fs::create_dir_all(&root).unwrap();
    let file=root.join("impact_horizon_native.hmod"); fs::copy(original,&file).unwrap();
    let mut mods=Mods::load(std::slice::from_ref(&file), |_|true, |_|false);
    assert_eq!(mods.entries()[0].status,Status::Running);
    assert!(!mods.impact_native_running(), "id alone must not grant authority");
    assert!(mods.impact_native_request("native_visual").is_none(), "untrusted mod cannot drive visuals");
    assert!(!mods.impact_native_trust(&root.join("unrelated.hmod")));
    assert!(mods.impact_native_trust(&file));
    assert!(mods.impact_native_running());
    assert!(mods.impact_native_owner(0));
    assert_eq!(mods.keys_of(0).len(), 22);
    let mut player=default_player(); player.x=1.0; player.y=70.0; player.z=2.0;
    player.dimension="minecraft:overworld".into();
    mods.set_snapshot(Snapshot {player:Some(player),gui_size:(640.0,360.0),..Snapshot::default()});
    let token="0123456789abcdef0123456789abcdef";
    let status=format!("v1|{token}|1|1|0|0|0|0|none|minecraft:overworld|0|0|0|0|none|0|0|0");
    let config="v1|0|0|0|3.0|0.2|hostile|diamond|32|0|48|0|0|0|0";
    assert!(mods.impact_native_publish("bridge_status",status.into_bytes()));
    assert!(mods.impact_native_publish("native_status",config.as_bytes().to_vec()));
    assert!(!mods.impact_native_publish("bridge_request",b"forged".to_vec()));
    assert!(!mods.impact_native_publish("native_visual",b"forged".to_vec()));
    assert!(!mods.impact_native_publish("native_status",vec![0;4097]));
    mods.frame(0.3);mods.key("key.keyboard.f8",true);mods.frame(0.01);mods.key("key.keyboard.f8",false);
    assert!(mods.drain().iter().any(|o|matches!(o,Output::OpenScreen{owner:0})));
    assert_eq!(label(&mods,53),"Find: OFF");
    assert!(label(&mods,82).starts_with("XRay OFF"));
    mods.ui_select(0,53,0);
    let request=String::from_utf8(mods.impact_native_request("bridge_request").unwrap()).unwrap();
    assert_eq!(request,format!("v1|{token}|1|ore|1"));
    let disk=root.join(".horizon-mod-data/impact_horizon_native/bridge_request");
    assert!(!disk.exists(),"native requests must never be written to disk");
    let status=format!("v1|{token}|2|1|1|0|0|1|diamond|minecraft:overworld|-2|65|9|1|zombie|1|70|3");
    let radar=format!("v1|{token}|minecraft:overworld|0|3.00,5.00,1;4.00,6.00,0");
    mods.impact_native_publish("bridge_status",status.into_bytes());
    mods.impact_native_publish("bridge_radar",radar.into_bytes());
    mods.frame(0.3);assert_eq!(label(&mods,53),"Find: ON");
    mods.command("ih map on");mods.tick(1);assert_eq!(mods.hud_shapes().count(),2);
    assert!(mods.command("ih arc on"));
    assert!(mods.command("ih espore on"));
    assert!(mods.command("ih espmobs on"));
    assert!(mods.command("ih glow on"));
    assert!(mods.command("ih aimwalls on"));
    assert!(mods.rebind(0,"ih_xray","key.keyboard.p"));
    mods.key("key.keyboard.p",true);mods.frame(0.3);mods.key("key.keyboard.p",false);
    assert!(mods.command("ih tracers on"));
    assert!(mods.command("ih xrayfilter iron off"));
    assert!(mods.command("ih xrayblock stone on"));
    mods.frame(0.3);
    let visual=String::from_utf8(mods.impact_native_request("native_visual").unwrap()).unwrap();
    assert_eq!(visual,format!("v4|{token}|1|1|1|1|1|1|1|8189|minecraft:stone=1"));
    assert!(!root.join(".horizon-mod-data/impact_horizon_native/native_visual").exists(),"visual state must not be written to disk");
    let lines:Vec<_>=mods.panels().flat_map(|p|p.lines.iter().map(|l|l.text.clone())).collect();
    assert!(lines.iter().any(|s|s.starts_with("Ore diamond: -2 / 65 / 9")),"{lines:?}");
    assert!(lines.iter().any(|s|s.starts_with("Mob radar: 1 | zombie")),"{lines:?}");
    assert_eq!(label(&mods,89),"Свои");
    assert_eq!(label(&mods,90),"Очистить список");
    mods.ui_select(0,89,0);
    let friend=String::from_utf8(mods.impact_native_request("native_request").unwrap()).unwrap();
    assert_eq!(friend,format!("v1|{token}|1|friends|select"));
    assert!(mods.drain().iter().any(|o|matches!(o,Output::CloseScreen{owner:0})));
    mods.ui_closed(0);
    mods.key("key.keyboard.f8",true);mods.frame(0.1);mods.key("key.keyboard.f8",false);
    assert_eq!(label(&mods,76),"Range 3.0");mods.ui_select(0,76,0);
    let config_req=String::from_utf8(mods.impact_native_request("native_request").unwrap()).unwrap();
    assert_eq!(config_req,format!("v1|{token}|2|range|3.5"));
    mods.command("ih goto 4 70 5");
    let nav=String::from_utf8(mods.impact_native_request("native_request").unwrap()).unwrap();
    assert_eq!(nav,format!("v1|{token}|3|goto|4|70|5"));
    for _ in 0..6 {mods.frame(0.9);}
    assert!(mods.command("ih attack on"));
    let out=mods.drain();
    assert!(out.iter().any(|x|matches!(x,Output::Feedback{error:true,..})),"{out:?}");
    assert_eq!(mods.entries()[0].status,Status::Running);
    drop(mods);fs::remove_dir_all(root).unwrap();
    println!("PASS actual Wasmtime: authenticated session-tagged v4 visuals, 22 bindings, one-screen menu, navigation, radar, stale heartbeat, no disk bridge");
}
