#![windows_subsystem = "windows"]

use bevy::prelude::*;

fn main() {
    App::new().add_plugins(foddian_jam::GamePlugin).run();
}
