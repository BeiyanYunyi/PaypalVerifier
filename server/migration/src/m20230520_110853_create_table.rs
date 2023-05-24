use sea_orm_migration::prelude::*;

#[derive(DeriveMigrationName)]
pub struct Migration;

#[async_trait::async_trait]
impl MigrationTrait for Migration {
  async fn up(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    // Replace the sample below with your own migration scripts
    manager
      .create_table(
        Table::create()
          .table(Record::Table)
          .if_not_exists()
          .col(ColumnDef::new(Record::OrderId).string_len(32).not_null())
          .col(
            ColumnDef::new(Record::AuthorizationId)
              .string_len(32)
              .not_null(),
          )
          .col(ColumnDef::new(Record::CaptureId).string_len(32))
          .col(ColumnDef::new(Record::CreateTime).string_len(24))
          .col(ColumnDef::new(Record::PayerEmail).text().not_null())
          .col(
            ColumnDef::new(Record::PayerId)
              .string_len(32)
              .primary_key()
              .not_null(),
          )
          .col(
            ColumnDef::new(Record::Used)
              .boolean()
              .not_null()
              .default(false),
          )
          .to_owned(),
      )
      .await?;
    manager
      .create_index(
        Index::create()
          .table(Record::Table)
          .unique()
          .name("payer_id_unique")
          .col(Record::PayerId)
          .if_not_exists()
          .to_owned(),
      )
      .await
  }

  async fn down(&self, manager: &SchemaManager) -> Result<(), DbErr> {
    // Replace the sample below with your own migration scripts
    manager
      .drop_table(Table::drop().table(Record::Table).to_owned())
      .await
  }
}

/// Learn more at https://docs.rs/sea-query#iden
#[derive(Iden)]
enum Record {
  Table,
  AuthorizationId,
  CaptureId,
  CreateTime,
  OrderId,
  PayerEmail,
  PayerId,
  Used,
}
