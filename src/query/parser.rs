// SQL Parser
// This module converts SQL strings into structured queries
// We use the sqlparser crate to handle the complex SQL grammar

use crate::storage::{Column, DataType, Schema, Value};
use anyhow::{anyhow, Result};
use sqlparser::ast::{
    BinaryOperator, DataType as SqlDataType, Expr, Select, SetExpr, Statement,
    TableFactor, Value as SqlValue,
};
use sqlparser::dialect::GenericDialect;
use sqlparser::parser::Parser;

/// Represents the different types of queries we support
#[derive(Debug)]
pub enum Query {
    /// CREATE TABLE tablename (col1 TYPE, col2 TYPE, ...)
    CreateTable {
        name: String,
        schema: Schema,
    },
    /// INSERT INTO tablename VALUES (val1, val2, ...)
    Insert {
        table_name: String,
        values: Vec<Value>,
    },
    /// SELECT * FROM tablename [WHERE column = value]
    Select {
        table_name: String,
        where_clause: Option<WhereClause>,
    },
    /// UPDATE tablename SET column = value WHERE column = value
    Update {
        table_name: String,
        set_column: String,
        set_value: Value,
        where_clause: WhereClause,
    },
    /// DELETE FROM tablename WHERE column = value
    Delete {
        table_name: String,
        where_clause: WhereClause,
    },
    /// CREATE INDEX ON tablename (column)
    CreateIndex {
        table_name: String,
        column_name: String,
    },
}

/// Represents a WHERE clause (simplified - only supports single conditions)
#[derive(Debug, Clone)]
pub struct WhereClause {
    pub column: String,
    pub value: Value,
}

/// The query parser
pub struct QueryParser;

impl QueryParser {
    /// Parse a SQL string into a Query
    /// This is the main entry point for parsing SQL
    pub fn parse(sql: &str) -> Result<Query> {
        // The sqlparser crate handles the complex SQL grammar
        let dialect = GenericDialect {};
        let ast = Parser::parse_sql(&dialect, sql)
            .map_err(|e| anyhow!("SQL parsing error: {}", e))?;

        // We only support single statements for simplicity
        if ast.len() != 1 {
            return Err(anyhow!("Only single statements are supported"));
        }

        let statement = &ast[0];

        // Match on the statement type
        // This is Rust's pattern matching - very powerful!
        match statement {
            Statement::CreateTable(create_table) => {
                Self::parse_create_table(create_table)
            }
            Statement::Insert(insert) => Self::parse_insert(insert),
            Statement::Query(query) => Self::parse_select(query),
            Statement::Update { table, assignments, selection, .. } => {
                Self::parse_update(table, assignments, selection)
            }
            Statement::Delete(delete) => Self::parse_delete(delete),
            Statement::CreateIndex(create_index) => {
                Self::parse_create_index(create_index)
            }
            _ => Err(anyhow!("Unsupported SQL statement")),
        }
    }

    /// Parse CREATE TABLE statement
    fn parse_create_table(
        create_table: &sqlparser::ast::CreateTable,
    ) -> Result<Query> {
        let table_name = create_table.name.to_string();
        let mut columns = Vec::new();

        for column_def in &create_table.columns {
            let name = column_def.name.to_string();
            let data_type = Self::parse_data_type(&column_def.data_type)?;

            // Check for PRIMARY KEY constraint
            let primary_key = column_def
                .options
                .iter()
                .any(|opt| matches!(opt.option, sqlparser::ast::ColumnOption::Unique { is_primary: true, .. }));

            // Check for NOT NULL constraint
            let nullable = !column_def
                .options
                .iter()
                .any(|opt| matches!(opt.option, sqlparser::ast::ColumnOption::NotNull));

            columns.push(Column {
                name,
                data_type,
                primary_key,
                nullable,
            });
        }

        Ok(Query::CreateTable {
            name: table_name,
            schema: Schema::new(columns),
        })
    }

    /// Parse INSERT statement
    fn parse_insert(insert: &sqlparser::ast::Insert) -> Result<Query> {
        let table_name = match &insert.table_name {
            sqlparser::ast::ObjectName(idents) => {
                idents.iter().map(|i| i.value.clone()).collect::<Vec<_>>().join(".")
            }
        };

        // We only support simple VALUES clause
        let values = match &insert.source {
            Some(source) => match source.body.as_ref() {
                SetExpr::Values(values) => {
                    if values.rows.is_empty() {
                        return Err(anyhow!("No values provided"));
                    }

                    // Take the first row (we only support single row inserts)
                    Self::parse_values(&values.rows[0])?
                }
                _ => return Err(anyhow!("Unsupported INSERT format")),
            },
            None => return Err(anyhow!("No values provided")),
        };

        Ok(Query::Insert { table_name, values })
    }

    /// Parse SELECT statement
    fn parse_select(query: &sqlparser::ast::Query) -> Result<Query> {
        let select = match query.body.as_ref() {
            SetExpr::Select(select) => select,
            _ => return Err(anyhow!("Unsupported SELECT format")),
        };

        // Extract table name
        let table_name = Self::extract_table_name(select)?;

        // Parse WHERE clause if present
        let where_clause = if let Some(selection) = &select.selection {
            Some(Self::parse_where_clause(selection)?)
        } else {
            None
        };

        Ok(Query::Select {
            table_name,
            where_clause,
        })
    }

    /// Parse UPDATE statement
    fn parse_update(
        table: &sqlparser::ast::TableWithJoins,
        assignments: &[sqlparser::ast::Assignment],
        selection: &Option<Expr>,
    ) -> Result<Query> {
        // Extract table name
        let table_name = match &table.relation {
            TableFactor::Table { name, .. } => {
                name.0.iter().map(|i| i.value.clone()).collect::<Vec<_>>().join(".")
            }
            _ => return Err(anyhow!("Unsupported table reference")),
        };

        // We only support single column updates
        if assignments.len() != 1 {
            return Err(anyhow!("Only single column updates are supported"));
        }

        let assignment = &assignments[0];
        let set_column = match &assignment.target {
            sqlparser::ast::AssignmentTarget::ColumnName(name) => {
                name.0.iter().map(|i| i.value.clone()).collect::<Vec<_>>().join(".")
            }
            _ => return Err(anyhow!("Unsupported assignment target")),
        };
        let set_value = Self::parse_value(&assignment.value)?;

        // WHERE clause is required for updates (safety feature)
        let where_clause = match selection {
            Some(expr) => Self::parse_where_clause(expr)?,
            None => return Err(anyhow!("UPDATE requires WHERE clause")),
        };

        Ok(Query::Update {
            table_name,
            set_column,
            set_value,
            where_clause,
        })
    }

    /// Parse DELETE statement
    fn parse_delete(delete: &sqlparser::ast::Delete) -> Result<Query> {
        // Extract table name
        let table_name = match &delete.tables.first() {
            Some(table) => {
                table.0.iter().map(|i| i.value.clone()).collect::<Vec<_>>().join(".")
            }
            None => return Err(anyhow!("No table specified")),
        };

        // WHERE clause is required for deletes (safety feature)
        let where_clause = match &delete.selection {
            Some(expr) => Self::parse_where_clause(expr)?,
            None => return Err(anyhow!("DELETE requires WHERE clause")),
        };

        Ok(Query::Delete {
            table_name,
            where_clause,
        })
    }

    /// Parse CREATE INDEX statement
    fn parse_create_index(
        create_index: &sqlparser::ast::CreateIndex,
    ) -> Result<Query> {
        let table_name = create_index.table_name.to_string();

        // We only support single column indexes
        if create_index.columns.len() != 1 {
            return Err(anyhow!("Only single column indexes are supported"));
        }

        let column_name = create_index.columns[0].to_string();

        Ok(Query::CreateIndex {
            table_name,
            column_name,
        })
    }

    /// Helper: Parse data type
    fn parse_data_type(sql_type: &SqlDataType) -> Result<DataType> {
        match sql_type {
            SqlDataType::Int(_) | SqlDataType::Integer(_) | SqlDataType::BigInt(_) => {
                Ok(DataType::Integer)
            }
            SqlDataType::Float(_) | SqlDataType::Double | SqlDataType::Real => {
                Ok(DataType::Float)
            }
            SqlDataType::Text | SqlDataType::Varchar(_) | SqlDataType::String(_) => {
                Ok(DataType::Text)
            }
            SqlDataType::Boolean => Ok(DataType::Boolean),
            _ => Err(anyhow!("Unsupported data type: {:?}", sql_type)),
        }
    }

    /// Helper: Parse a list of SQL values
    fn parse_values(exprs: &[Expr]) -> Result<Vec<Value>> {
        exprs.iter().map(Self::parse_value).collect()
    }

    /// Helper: Parse a single SQL value
    fn parse_value(expr: &Expr) -> Result<Value> {
        match expr {
            Expr::Value(SqlValue::Number(n, _)) => {
                if n.contains('.') {
                    // Parse as float and store as integer (multiply by 1000 for precision)
                    let float_val: f64 = n.parse()?;
                    Ok(Value::Float((float_val * 1000.0) as i64))
                } else {
                    Ok(Value::Integer(n.parse()?))
                }
            }
            Expr::Value(SqlValue::SingleQuotedString(s))
            | Expr::Value(SqlValue::DoubleQuotedString(s)) => Ok(Value::Text(s.clone())),
            Expr::Value(SqlValue::Boolean(b)) => Ok(Value::Boolean(*b)),
            Expr::Value(SqlValue::Null) => Ok(Value::Null),
            _ => Err(anyhow!("Unsupported value expression: {:?}", expr)),
        }
    }

    /// Helper: Extract table name from SELECT
    fn extract_table_name(select: &Select) -> Result<String> {
        if select.from.is_empty() {
            return Err(anyhow!("No table specified in SELECT"));
        }

        match &select.from[0].relation {
            TableFactor::Table { name, .. } => {
                Ok(name.0.iter().map(|i| i.value.clone()).collect::<Vec<_>>().join("."))
            }
            _ => Err(anyhow!("Unsupported table reference")),
        }
    }

    /// Helper: Parse WHERE clause
    /// We only support simple equality conditions: column = value
    fn parse_where_clause(expr: &Expr) -> Result<WhereClause> {
        match expr {
            Expr::BinaryOp { left, op, right } => {
                if !matches!(op, BinaryOperator::Eq) {
                    return Err(anyhow!("Only = operator is supported in WHERE clause"));
                }

                let column = match left.as_ref() {
                    Expr::Identifier(ident) => ident.value.clone(),
                    _ => return Err(anyhow!("Expected column name in WHERE clause")),
                };

                let value = Self::parse_value(right)?;

                Ok(WhereClause { column, value })
            }
            _ => Err(anyhow!("Unsupported WHERE clause format")),
        }
    }
}
