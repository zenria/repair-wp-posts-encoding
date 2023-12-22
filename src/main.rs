use std::env;

use chardetng::EncodingDetector;
use futures::TryStreamExt;
use sqlx::{MySqlPool, QueryBuilder, Row};

const TABLE: &str = "wp_posts";
const COLUMNS: &[&str] = &[
    "post_content",
    "post_title",
    "post_excerpt",
    "post_content_filtered",
];
const COLUMNS_TYPE: &[&str] = &["longtext", "text", "text", "longtext"];

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // autoload env from .env file
    let _ = dotenvy::dotenv();

    let pool = MySqlPool::connect(&env::var("DATABASE_URL")?).await?;

    for (column, column_type) in COLUMNS.iter().zip(COLUMNS_TYPE.iter()) {
        println!("******** {column} ********");
        println!("Converting to binary");
        let to_binary_query = format!(
            "ALTER TABLE {TABLE} MODIFY {column} {column_type} CHARACTER SET binary NOT NULL"
        );
        QueryBuilder::new(&to_binary_query)
            .build()
            .execute(&pool)
            .await?;

        let get_binary_query = format!("SELECT ID, {column} FROM {TABLE}");
        let update_query = format!("UPDATE {TABLE} SET {column}=? WHERE ID=?");
        println!("Checking utf8 encoding");
        let mut rows = sqlx::query(&get_binary_query).fetch(&pool);

        while let Some(row) = rows.try_next().await? {
            let id: u64 = row.try_get("ID")?;
            let content: &[u8] = row.try_get(column)?;
            // check utf8 validity by trying to convert &[u8] to &str (no alloc required)
            if let Err(_) = std::str::from_utf8(content) {
                let mut detector = EncodingDetector::new();
                detector.feed(content, true);
                let detected = detector.guess(Some("fr".as_bytes()), false);

                println!(
                    "{column}:{id} has invalid utf8, let's repair it! len:{} encoding:{detected:?}",
                    content.len()
                );

                let fixed = detected.decode(content).0;
                sqlx::query(&update_query)
                    .bind(fixed)
                    .bind(id)
                    .execute(&pool)
                    .await?;
                println!("{column}:{id} fixed!",);
            }
        }
        let to_utf8 = format!(
            "ALTER TABLE {TABLE} MODIFY {column} {column_type} CHARACTER SET utf8mb4 NOT NULL COLLATE utf8mb4_general_ci"
        );
        println!("Convert column to utf8!");
        sqlx::query(&to_utf8).execute(&pool).await?;
    }

    Ok(())
}
