//! Integration test: full E2E flow, mirroring the Python version that passed.
//! Simulates a Lutris install result: prefix + game + bundled switcher + settings.dat.

use spfl27::core::*;
use std::fs;
use std::path::{Path, PathBuf};

fn tmp(tag: &str) -> PathBuf {
    let p = std::env::temp_dir().join(format!("spfl27-e2e-{}-{tag}", std::process::id()));
    let _ = fs::remove_dir_all(&p);
    fs::create_dir_all(&p).unwrap();
    p
}

struct Fake {
    root: PathBuf,
    prefix: PathBuf,
    game: PathBuf,
    konami: PathBuf,
}

fn build(root: &Path) -> Fake {
    let prefix = root.join("sp-football-life-2027");
    let game = prefix
        .join("drive_c")
        .join("Program Files (x86)")
        .join(GAME_FOLDER_NAME);
    fs::create_dir_all(game.join("Data")).unwrap();
    fs::create_dir_all(game.join("download")).unwrap();
    fs::create_dir_all(game.join("SiderAddons/modules")).unwrap();
    fs::write(game.join("FL_2027.exe"), b"fake").unwrap();
    fs::write(game.join("SiderAddons/sider.exe"), b"fake").unwrap();
    fs::write(game.join("SiderAddons/modules/aspectratio.lua"), b"-- fake").unwrap();
    fs::write(game.join(SWITCHER_EXE), b"fake").unwrap();
    fs::write(game.join(SETTINGS_EXE), b"fake").unwrap();
    // bundled game switcher: GP + Controller + Classics
    for (idx, sm) in SWITCH_MAP.iter().enumerate() {
        let (sub, fname) = (sm.1, sm.3);
        let label = format!("cpk-{idx}");
        let v = format!("VAR{label}");
        fs::create_dir_all(game.join("switcher").join(sub).join(&v)).unwrap();
        fs::write(
            game.join("switcher").join(sub).join(&v).join(fname),
            label.as_bytes(),
        )
        .unwrap();
    }
    let konami =
        prefix.join("drive_c/users/deck/Documents/KONAMI/eFootball PES 2021 SEASON UPDATE");
    fs::create_dir_all(&konami).unwrap();
    fs::write(konami.join("settings.dat"), vec![0u8; 64]).unwrap();
    Fake {
        root: root.to_path_buf(),
        prefix,
        game,
        konami,
    }
}

#[test]
fn e2e_full_flow() {
    let root = tmp("full");
    let f = build(&root);

    // 1. prefix detected
    assert_eq!(prefix_of(&f.game), Some(f.prefix.clone()));

    // 2. status reads the filesystem
    let st = game_status(Some(&f.game));
    assert!(
        st.game && st.sider && st.aspect && st.settings,
        "status: {st:?}"
    );

    // 3. switcher source = bundled game folder
    let src = cpk_source(&f.game).expect("bundled game switcher must exist");
    assert_eq!(src, f.game.join("switcher"));

    // 4. apply switcher for all three labels -> target file + dataSP log
    for sm in SWITCH_MAP.iter() {
        let label = sm.0;
        let opts = list_options(&src, label);
        let choice = opts.first().expect("minimal satu opsi").clone();
        let msg = apply_switch(&f.game, &src, label, &choice).unwrap();
        assert!(msg.contains(&format!("{label} -> {choice}")), "{msg}");
        assert_eq!(read_current(&f.game, label), choice);
    }
    assert_eq!(
        fs::read(f.game.join("Data/dt18_all.cpk")).unwrap(),
        b"cpk-0"
    );
    assert!(f.game.join("download/SMK_Addcontrol.cpk").is_file());
    assert!(f.game.join("download/08_smkdb_ulk.cpk").is_file());

    // 5. settings.dat tulis + baca + backup
    let dat = f.konami.join("settings.dat");
    let msg = write_settings(&dat, "XInput", "Full Screen", "Enable2").unwrap();
    assert!(msg.contains("7708"), "{msg}");
    let s = read_settings(&dat).unwrap();
    assert_eq!(s.controller, "XInput");
    assert_eq!(s.screen, "Full Screen");
    assert_eq!(s.vsync, "Enable2");
    assert!(f.konami.join("settings.dat.backup").is_file());
    // idempotent
    assert!(write_settings(&dat, "XInput", "Full Screen", "Enable2")
        .unwrap()
        .contains("No change"));
    // change to other values
    write_settings(&dat, "DirectInput", "Windowed", "Disabled").unwrap();
    assert_eq!(fs::read(&dat).unwrap()[8..10], [0x66, 0x02]);

    // 6. backup & restore saves (KONAMI folder used as root)
    let konami_root = f.konami.parent().unwrap().to_path_buf();
    let sub = f.konami.file_name().unwrap().to_string_lossy().into_owned();
    let dest = backup_saves(&konami_root, &root.join("backups")).unwrap();
    assert!(
        dest.join(&sub).join("settings.dat").is_file(),
        "backup must contain settings.dat"
    );
    fs::remove_dir_all(&konami_root).unwrap();
    restore_saves(&dest, &konami_root).unwrap();
    assert!(
        konami_root.join(&sub).join("settings.dat").is_file(),
        "restore must bring back the save"
    );
    assert_eq!(
        fs::read(konami_root.join(&sub).join("settings.dat")).unwrap()[8..10],
        [0x66, 0x02],
        "settings.dat must match the pre-backup content"
    );

    // 7. Steam shortcut for --play mode
    let desktop = f.root.join("Desktop/SP Football Life 2027.desktop");
    make_game_desktop(&f.game, &desktop).unwrap();
    let txt = fs::read_to_string(&desktop).unwrap();
    assert!(txt.contains("--play"), "{txt}");
    assert!(txt.contains(GAME_FOLDER_NAME), "{txt}");

    // 8. Lutris cache (may be 0 items on this machine, but must not error)
    let _ = clear_lutris_cache();
}

#[test]
fn e2e_missing_switcher_gives_guidance() {
    let root = tmp("noswitcher");
    let prefix = root.join("pfx");
    let game = prefix
        .join("drive_c")
        .join("Program Files (x86)")
        .join(GAME_FOLDER_NAME);
    fs::create_dir_all(&game).unwrap();
    fs::write(game.join("FL_2027.exe"), b"x").unwrap();
    // no switcher folder → cpk_source None (UI shows guidance, no crash)
    assert!(cpk_source(&game).is_none());
    let st = game_status(Some(&game));
    assert!(st.game && !st.sider);
}

#[test]
fn e2e_path_traversal_rejected() {
    let root = tmp("traversal");
    let f = build(&root);
    let src = f.game.join("switcher");
    for bad in ["../../../../etc", "..", "/etc/passwd", ""] {
        let e = apply_switch(&f.game, &src, "GP", bad).unwrap_err();
        assert!(
            e.to_string().contains("Invalid choice"),
            "slipped through: {bad:?}"
        );
    }
}

#[test]
fn e2e_corrupt_settings_not_written() {
    let root = tmp("corrupt");
    let f = build(&root);
    let dat = f.konami.join("settings.dat");
    fs::write(&dat, b"\x77\x08").unwrap(); // only 2 bytes
    assert!(read_settings(&dat).is_err());
    let before = fs::read(&dat).unwrap();
    assert!(write_settings(&dat, "XInput", "Full Screen", "Enable1").is_err());
    assert_eq!(
        fs::read(&dat).unwrap(),
        before,
        "corrupt file must not be modified"
    );
    assert!(
        !f.konami.join("settings.dat.backup").exists(),
        "backup must not be created"
    );
}

#[test]
fn e2e_launch_cmd_menunjuk_ke_exe_benar() {
    let root = tmp("launch");
    let f = build(&root);
    let cmd = build_launch_cmd(&f.game);
    assert!(cmd[0].ends_with("wine"), "{cmd:?}");
    assert!(cmd[1].ends_with("FL_2027.exe"), "{cmd:?}");

    // if only the .NET launcher exists, use it
    fs::remove_file(f.game.join("FL_2027.exe")).unwrap();
    fs::write(f.game.join("FL 2027 start.exe"), b"x").unwrap();
    let cmd2 = build_launch_cmd(&f.game);
    assert!(cmd2[1].ends_with("FL 2027 start.exe"), "{cmd2:?}");
}

#[test]
fn e2e_switcher_command_points_to_game_tool() {
    let root = tmp("switcher-cmd");
    let f = build(&root);
    let cmd = build_switcher_cmd(&f.game);
    assert!(cmd[0].ends_with("wine"), "{cmd:?}");
    assert!(cmd[1].ends_with(SWITCHER_EXE), "{cmd:?}");
    assert!(f.game.join(SWITCHER_EXE).is_file());
}

#[test]
fn e2e_settings_command_points_to_game_tool() {
    let root = tmp("settings-cmd");
    let f = build(&root);
    let cmd = build_settings_cmd(&f.game);
    assert!(cmd[0].ends_with("wine"), "{cmd:?}");
    assert!(cmd[1].ends_with(SETTINGS_EXE), "{cmd:?}");
    assert!(f.game.join(SETTINGS_EXE).is_file());
}

#[test]
fn e2e_detects_spgame_football_life_save_path() {
    let root = tmp("spgame-save");
    let prefix = root.join("pfx");
    let game = prefix
        .join("drive_c")
        .join("Program Files (x86)")
        .join(GAME_FOLDER_NAME);
    let save_dir = prefix.join("drive_c/users/deck/Documents/SPGAME/Football Life");
    fs::create_dir_all(game.join("Data")).unwrap();
    fs::create_dir_all(save_dir.join("2027/save")).unwrap();
    fs::write(game.join("FL_2027.exe"), b"x").unwrap();
    fs::write(save_dir.join("settings.dat"), vec![0u8; 64]).unwrap();

    let found = find_settings_dat(&game).expect("SPGAME save must be detected");
    assert_eq!(found, save_dir.join("settings.dat"));
    assert_eq!(find_konami_dir(&game), Some(save_dir));
}
