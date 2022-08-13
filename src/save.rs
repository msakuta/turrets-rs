use crate::{
    tower::{
        spawn_healer, spawn_missile_tower, spawn_shotgun, spawn_turret, Healer, MissileTower,
        Shotgun, Tower, TowerInitBundle, TowerLevel, TowerScore,
    },
    Health, Position, Rotation, Scoreboard,
};
use bevy::prelude::*;
use serde_json::{from_str, from_value, json, Value};

#[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
const WASM_SAVE_KEY: &str = "turret-rs/save";

pub(crate) struct SaveGameEvent;

#[derive(Debug)]
struct MyError(String);

impl<T: std::error::Error> From<T> for MyError {
    fn from(e: T) -> Self {
        Self(e.to_string())
    }
}

pub(crate) fn save_game(
    mut reader: EventReader<SaveGameEvent>,
    query: Query<
        (
            &Position,
            &Rotation,
            &TowerScore,
            &TowerLevel,
            &Health,
            Option<&Shotgun>,
            Option<&Healer>,
            Option<&MissileTower>,
        ),
        With<Tower>,
    >,
    scoreboard: Res<Scoreboard>,
) {
    for _e in reader.iter() {
        println!("Save event");

        match (|| -> Result<(), MyError> {
            let json_towers = query.iter().map(|(position, rotation, tower_score, tower_level, health, shotgun, healer, missile_tower)| -> Result<serde_json::Value, MyError>{
                Ok(json!({
                    "type": if shotgun.is_some() { "Shotgun" } else if healer.is_some() { "Healer" } else if missile_tower.is_some() { "MissileTower" } else { "Turret"},
                    "tower_score": tower_score,
                    "tower_level": tower_level,
                    "position": position,
                    "rotation": rotation,
                    "health": health,
                }))
            }).collect::<Result<serde_json::Value, MyError>>()?;

            let json_container = json!({
                "scoreboard": &*scoreboard,
                "towers": json_towers,
            });

            #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
            std::fs::write("save.json", serde_json::to_string(&json_container)?)?;

            #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
            {
                let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();

                local_storage
                    .set_item(WASM_SAVE_KEY, &serde_json::to_string(&json_container)?)
                    .unwrap();
            }

            Ok(())
        })() {
            Ok(()) => println!("Save succeeded"),
            Err(e) => println!("Save failed!: {e:?}"),
        }
    }
}

macro_rules! _unwrap_or_continue {
    {$e:expr} => {
        if let Some(e) = $e {
            e
        } else {
            continue;
        }
    }
}

macro_rules! take_or_continue {
    {$tower:expr, $name:literal} => {
        if let Some(e) = $tower.get_mut($name).map(|p| p.take()) {
            e
        } else {
            println!("Field {} was not found", $name);
            continue;
        }
    }
}

pub(crate) fn load_game(
    commands: &mut Commands,
    asset_server: &Res<AssetServer>,
    scoreboard: &mut Scoreboard,
) {
    #[cfg(not(all(target_arch = "wasm32", target_os = "unknown")))]
    let json_str = if let Ok(json_str) = std::fs::read_to_string("save.json") {
        json_str
    } else {
        println!("Save file was not found!");
        return;
    };

    #[cfg(all(target_arch = "wasm32", target_os = "unknown"))]
    let json_str = {
        let local_storage = web_sys::window().unwrap().local_storage().unwrap().unwrap();

        if let Some(value) = local_storage.get_item(WASM_SAVE_KEY).unwrap() {
            value
        } else {
            return;
        }
    };

    println!("json: {json_str}");
    match (|| -> Result<(), MyError> {
        let mut json_container: Value = from_str(&json_str)?;

        let json_scoreboard = json_container
            .get_mut("scoreboard")
            .map(|j| j.take())
            .ok_or_else(|| MyError("scoreboard is mandatory".to_string()))?;
        *scoreboard = from_value(json_scoreboard)?;

        println!("json: {json_container:?}");
        if let Some(Value::Array(arr)) = json_container.get_mut("towers").map(|t| t.take()) {
            for mut tower in arr {
                let position = take_or_continue!(tower, "position");
                let rotation = take_or_continue!(tower, "rotation");
                let health = take_or_continue!(tower, "health");
                let tower_score = take_or_continue!(tower, "tower_score");
                let tower_level = take_or_continue!(tower, "tower_level");
                let tower_type = if let Some(Value::String(s)) = tower.get("type") {
                    s
                } else {
                    println!("No type defined");
                    continue;
                };

                let bundle = TowerInitBundle {
                    health: Some(serde_json::from_value(health)?),
                    tower_score: Some(serde_json::from_value(tower_score)?),
                    tower_level: Some(serde_json::from_value(tower_level)?),
                };

                match tower_type as _ {
                    "Turret" => {
                        spawn_turret(
                            commands,
                            asset_server,
                            serde_json::from_value(position)?,
                            serde_json::from_value(rotation)?,
                            bundle,
                        );
                    }
                    "Shotgun" => {
                        spawn_shotgun(
                            commands,
                            asset_server,
                            serde_json::from_value(position)?,
                            serde_json::from_value(rotation)?,
                            bundle,
                        );
                    }
                    "Healer" => {
                        spawn_healer(
                            commands,
                            asset_server,
                            serde_json::from_value(position)?,
                            serde_json::from_value(rotation)?,
                            bundle,
                        );
                    }
                    "MissileTower" => {
                        spawn_missile_tower(
                            commands,
                            asset_server,
                            serde_json::from_value(position)?,
                            serde_json::from_value(rotation)?,
                            bundle,
                        );
                    }
                    _ => println!("Unrecognized type!"),
                }
            }
        }
        Ok(())
    })() {
        Ok(()) => println!("Loaded from file successfully"),
        Err(e) => println!("Load error: {e:?}"),
    }
}
