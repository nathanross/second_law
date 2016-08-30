mod atpath;
mod fixtures;
mod cmdresult;
mod ucommand;
mod scene;
mod common;
mod settings;

pub use atpath::AtPath;
pub use ucommand::UCommand;
pub use scene::Scene;
pub use cmdresult::CmdResult;


#[macro_export]
macro_rules! new_scene {
    () => ({
        use second_law;
        if cfg!(target_os = "windows") {
            second_law::Scene::new(format!("{}.exe", env!("CARGO_PKG_NAME")))
        } else {
            second_law::Scene::new(env!("CARGO_PKG_NAME"))
        }
    });
}
