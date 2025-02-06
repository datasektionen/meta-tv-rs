use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct AppErrorDto {
    pub msg: String,
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct ScreenDto {
    pub id: i32,
    pub name: String,
    pub position: i32,
}

#[cfg(feature = "entity")]
impl From<entity::screen::Model> for ScreenDto {
    fn from(screen: entity::screen::Model) -> Self {
        Self {
            id: screen.id,
            name: screen.name,
            position: screen.position,
        }
    }
}

#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct CreateScreenDto {
    pub name: String,
    pub position: i32,
}
