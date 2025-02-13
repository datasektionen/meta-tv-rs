pub use sea_orm_migration::prelude::*;

mod m20250206_144027_create_slides_tables;
mod m20250213_173223_add_published_slide_group;

pub struct Migrator;

#[async_trait::async_trait]
impl MigratorTrait for Migrator {
    fn migrations() -> Vec<Box<dyn MigrationTrait>> {
        vec![
            Box::new(m20250206_144027_create_slides_tables::Migration),
            Box::new(m20250213_173223_add_published_slide_group::Migration),
        ]
    }
}
