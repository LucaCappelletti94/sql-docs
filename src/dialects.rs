//! Handles the selection of the `SQL` dialect to use for parsing

use sqlparser::dialect::{
    AnsiDialect, BigQueryDialect, ClickHouseDialect, DatabricksDialect, Dialect, DuckDbDialect,
    GenericDialect, HiveDialect, MsSqlDialect, MySqlDialect, OracleDialect, PostgreSqlDialect,
    RedshiftSqlDialect, SQLiteDialect, SnowflakeDialect,
};

/// Dialects supported by this crate.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Dialects {
    /// ANSI SQL dialect
    Ansi,
    /// Google `BigQuery` SQL dialect
    BigQuery,
    /// `ClickHouse` SQL dialect
    ClickHouse,
    /// Databricks SQL dialect
    Databricks,
    /// `DuckDB` SQL dialect
    DuckDb,
    /// Generic SQL dialect
    Generic,
    /// Apache Hive SQL dialect
    Hive,
    /// Microsoft SQL Server (T-SQL) dialect
    MsSql,
    /// `MySQL` SQL dialect
    MySql,
    /// Oracle SQL dialect
    Oracle,
    /// `PostgreSQL` SQL dialect
    #[default]
    PostgreSql,
    /// Amazon Redshift SQL dialect
    RedshiftSql,
    /// `SQLite` SQL dialect
    SQLite,
    /// Snowflake SQL dialect
    Snowflake,
}

impl Dialects {
    /// Returns the dialect struct associated with the enum
    #[must_use]
    pub fn dialect(&self) -> &'static dyn Dialect {
        match self {
            Self::Ansi => &AnsiDialect {},
            Self::BigQuery => &BigQueryDialect {},
            Self::ClickHouse => &ClickHouseDialect {},
            Self::Databricks => &DatabricksDialect {},
            Self::DuckDb => &DuckDbDialect {},
            Self::Generic => &GenericDialect {},
            Self::Hive => &HiveDialect {},
            Self::MsSql => &MsSqlDialect {},
            Self::MySql => &MySqlDialect {},
            Self::Oracle => &OracleDialect {},
            Self::PostgreSql => &PostgreSqlDialect {},
            Self::RedshiftSql => &RedshiftSqlDialect {},
            Self::SQLite => &SQLiteDialect {},
            Self::Snowflake => &SnowflakeDialect {},
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;

    #[test]
    fn default_is_postgres() {
        assert_eq!(Dialects::default(), Dialects::PostgreSql);
    }

    fn assert_is<T: 'static>(d: Dialects) {
        let any = d.dialect() as &dyn Any;
        assert!(
            any.is::<T>(),
            "expected {:?} to map to {}, got a different dialect type",
            d,
            std::any::type_name::<T>(),
        );
    }

    #[test]
    fn dialect_maps_to_correct_concrete_type() {
        assert_is::<AnsiDialect>(Dialects::Ansi);
        assert_is::<BigQueryDialect>(Dialects::BigQuery);
        assert_is::<ClickHouseDialect>(Dialects::ClickHouse);
        assert_is::<DatabricksDialect>(Dialects::Databricks);
        assert_is::<DuckDbDialect>(Dialects::DuckDb);
        assert_is::<GenericDialect>(Dialects::Generic);
        assert_is::<HiveDialect>(Dialects::Hive);
        assert_is::<MsSqlDialect>(Dialects::MsSql);
        assert_is::<MySqlDialect>(Dialects::MySql);
        assert_is::<OracleDialect>(Dialects::Oracle);
        assert_is::<PostgreSqlDialect>(Dialects::PostgreSql);
        assert_is::<RedshiftSqlDialect>(Dialects::RedshiftSql);
        assert_is::<SQLiteDialect>(Dialects::SQLite);
        assert_is::<SnowflakeDialect>(Dialects::Snowflake);
    }

    #[test]
    fn dialect_reference_is_stable_for_same_variant() {
        let d = Dialects::PostgreSql;
        let p1 = std::ptr::from_ref::<dyn Dialect>(d.dialect());
        let p2 = std::ptr::from_ref::<dyn Dialect>(d.dialect());
        assert_eq!(p1, p2);
    }
}
