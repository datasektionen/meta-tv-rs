use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(SlideGroup::Table)
                    .add_column(boolean(SlideGroup::Published))
                    .to_owned(),
            )
            .await
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .alter_table(
                Table::alter()
                    .table(SlideGroup::Table)
                    .drop_column(SlideGroup::Published)
                    .to_owned(),
            )
            .await
    }
}

#[derive(DeriveIden)]
enum SlideGroup {
    Table,
    Published,
}
