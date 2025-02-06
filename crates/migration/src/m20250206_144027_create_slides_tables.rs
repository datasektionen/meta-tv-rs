use extension::postgres::Type;
use sea_orm::{EnumIter, Iterable};
use sea_orm_migration::{prelude::*, schema::*};

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
    async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .create_type(
                Type::create()
                    .as_enum(ContentType)
                    .values(ContentTypeVariants::iter())
                    .to_owned(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Screen::Table)
                    .if_not_exists()
                    .col(pk_auto(Screen::Id))
                    .col(string(Screen::Name))
                    .col(integer(Screen::Position))
                    .take(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(SlideGroup::Table)
                    .if_not_exists()
                    .col(pk_auto(SlideGroup::Id))
                    .col(string(SlideGroup::Title))
                    .col(integer(SlideGroup::Priority))
                    .col(boolean(SlideGroup::Hidden))
                    .col(string(SlideGroup::CreatedBy))
                    .col(timestamp(SlideGroup::StartDate))
                    .col(timestamp_null(SlideGroup::EndDate))
                    .col(timestamp_null(SlideGroup::ArchiveDate))
                    .take(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Slide::Table)
                    .if_not_exists()
                    .col(pk_auto(Slide::Id))
                    .col(integer(Slide::Position))
                    .col(integer(Slide::Group))
                    .col(timestamp_null(Slide::ArchiveDate))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-slide-group")
                            .from(Slide::Table, Slide::Group)
                            .to(SlideGroup::Table, SlideGroup::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .take(),
            )
            .await?;

        manager
            .create_table(
                Table::create()
                    .table(Content::Table)
                    .if_not_exists()
                    .col(pk_auto(Content::Id))
                    .col(integer(Content::Slide))
                    .col(integer(Content::Screen))
                    .col(enumeration(
                        Content::ContentType,
                        ContentType,
                        ContentTypeVariants::iter(),
                    ))
                    .col(string(Content::FilePath))
                    .col(timestamp_null(Content::ArchiveDate))
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-content-slide")
                            .from(Content::Table, Content::Slide)
                            .to(Slide::Table, Slide::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .foreign_key(
                        ForeignKey::create()
                            .name("fk-content-screen")
                            .from(Content::Table, Content::Screen)
                            .to(Screen::Table, Screen::Id)
                            .on_delete(ForeignKeyAction::Restrict)
                            .on_update(ForeignKeyAction::Cascade),
                    )
                    .take(),
            )
            .await?;

        Ok(())
    }

    async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
        manager
            .drop_table(Table::drop().table(Screen::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(SlideGroup::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Slide::Table).to_owned())
            .await?;
        manager
            .drop_table(Table::drop().table(Content::Table).to_owned())
            .await?;

        Ok(())
    }
}

#[derive(DeriveIden)]
enum Screen {
    Table,
    Id,
    Name,
    Position,
}

#[derive(DeriveIden)]
enum SlideGroup {
    Table,
    Id,
    Title,
    Priority,
    Hidden,
    CreatedBy,
    StartDate,
    EndDate,
    ArchiveDate,
}

#[derive(DeriveIden)]
enum Slide {
    Table,
    Id,
    Group,
    Position,
    ArchiveDate,
}

#[derive(DeriveIden)]
enum Content {
    Table,
    Id,
    Slide,
    Screen,
    #[allow(clippy::enum_variant_names)]
    ContentType,
    FilePath,
    ArchiveDate,
}

#[derive(DeriveIden)]
struct ContentType;

#[derive(DeriveIden, EnumIter)]
enum ContentTypeVariants {
    Image,
    Video,
    Html,
}
