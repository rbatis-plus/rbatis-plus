//! 基于 Tera 的模板引擎（对标 mybatis-plus-generator `VelocityTemplateEngine`）。

use crate::config::{GlobalConfig, PackageConfig, StrategyConfig};
use crate::query::TableInfo;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// 基于 Tera 的代码生成引擎（对标 Java `AbstractTemplateEngine` + `VelocityTemplateEngine`）。
///
/// 使用 Tera（Rust 版 Jinja2/FreeMarker）渲染代码模板。
pub struct TeraEngine {
    global_config: GlobalConfig,
    package_config: PackageConfig,
    strategy_config: StrategyConfig,
}

impl TeraEngine {
    /// 创建模板引擎。
    pub fn new(
        global_config: GlobalConfig,
        package_config: PackageConfig,
        strategy_config: StrategyConfig,
    ) -> Self {
        Self {
            global_config,
            package_config,
            strategy_config,
        }
    }

    /// 批量生成所有表的代码（对标 `AbstractTemplateEngine.batchOutput()`）。
    pub fn batch_output(&self, tables: &[TableInfo]) -> Result<Vec<PathBuf>, String> {        let mut generated_files = Vec::new();

        for table in tables {
            let files = self.output_table(table)?;
            generated_files.extend(files);
        }

        Ok(generated_files)
    }

    /// 为单个表生成所有代码文件。
    fn output_table(&self, table: &TableInfo) -> Result<Vec<PathBuf>, String> {
        let mut files = Vec::new();
        let context = self.build_context(table);

        // 生成 Entity
        let entity_path = self.output_entity(table, &context)?;
        files.push(entity_path);

        // 生成 Mapper
        if self.strategy_config.generate_mapper {
            let mapper_path = self.output_mapper(table, &context)?;
            files.push(mapper_path);
        }

        // 生成 Service
        if self.strategy_config.generate_service {
            let service_path = self.output_service(table, &context)?;
            files.push(service_path);
        }

        // 生成 Controller
        if self.strategy_config.generate_controller {
            let controller_path = self.output_controller(table, &context)?;
            files.push(controller_path);
        }

        Ok(files)
    }

    /// 构建模板上下文变量（对标 `AbstractTemplateEngine.getObjectMap()`）。
    fn build_context(&self, table: &TableInfo) -> HashMap<String, String> {
        let mut ctx = HashMap::new();

        let entity_name = table.entity_name();
        let module_name = table.module_name();
        let now = chrono::Local::now().format(&self.global_config.date_format).to_string();

        ctx.insert("entity_name".to_string(), entity_name.clone());
        ctx.insert("module_name".to_string(), module_name);
        ctx.insert("table_name".to_string(), table.name.clone());
        ctx.insert("table_comment".to_string(), table.comment.clone());
        ctx.insert("author".to_string(), self.global_config.author.clone());
        ctx.insert("date".to_string(), now);
        ctx.insert("entity_package".to_string(), self.package_config.entity_package());
        ctx.insert("mapper_package".to_string(), self.package_config.mapper_package());
        ctx.insert("service_package".to_string(), self.package_config.service_package());
        ctx.insert("controller_package".to_string(), self.package_config.controller_package());

        // Entity 字段信息（序列化为 JSON）
        if let Ok(fields_json) = serde_json::to_string(&table.fields) {
            ctx.insert("fields_json".to_string(), fields_json);
        }

        ctx
    }

    /// 生成 Entity 文件。
    fn output_entity(&self, table: &TableInfo, ctx: &HashMap<String, String>) -> Result<PathBuf, String> {
        let filename = format!("{}.rs", table.module_name());
        let output_dir = PathBuf::from(&self.global_config.output_dir).join(&self.package_config.entity);
        let output_path = output_dir.join(&filename);

        let content = self.render_entity(table, ctx)?;

        self.write_file(&output_path, &content)?;
        Ok(output_path)
    }

    /// 渲染 Entity 代码（对标 `entity.java.vm`）。
    fn render_entity(&self, table: &TableInfo, ctx: &HashMap<String, String>) -> Result<String, String> {
        let entity_name = ctx.get("entity_name").unwrap();
        let table_name = &table.name;
        let author = ctx.get("author").unwrap();
        let date = ctx.get("date").unwrap();
        let table_comment = &table.comment;

        let mut code = String::new();

        // 文件头注释
        code.push_str(&format!("//! {} — {}\n", entity_name, table_comment));
        code.push_str(&format!("//!\n//! 对应数据库表: `{}`\n", table_name));
        code.push_str(&format!("//! 生成时间: {}  作者: {}\n\n", date, author));

        // use 语句
        code.push_str("use serde::{Deserialize, Serialize};\n");

        // derive 宏
        if self.strategy_config.entity_lombok {
            code.push_str("#[derive(Debug, Clone, Serialize, Deserialize)]\n");
            code.push_str("#[derive(rbatis_plus_macros::TableName)]\n");
        }

        // #[table_name] 注解
        if self.strategy_config.entity.use_table_name_annotation {
            code.push_str(&format!("#[table_name = \"{}\"]\n", table_name));
        }

        code.push_str(&format!("pub struct {} {{\n", entity_name));

        // 字段
        for field in &table.fields {
            let prop_name = field.property_name();
            let comment = if field.comment.is_empty() {
                String::new()
            } else {
                format!(" /// {}", field.comment)
            };
            let serde_rename = if prop_name != field.name {
                format!("    #[serde(rename = \"{}\")]\n", field.name)
            } else {
                String::new()
            };

            // 主键注解
            if field.is_primary_key && self.strategy_config.entity.use_table_id_annotation {
                code.push_str("    #[table_id]\n");
            }

            code.push_str(&serde_rename);
            code.push_str(&format!("    pub {}: {},{}\n", prop_name, field.rust_type, comment));
        }

        code.push_str("}\n");

        Ok(code)
    }

    /// 生成 Mapper 文件。
    fn output_mapper(&self, table: &TableInfo, ctx: &HashMap<String, String>) -> Result<PathBuf, String> {
        let module_name = table.module_name();
        let entity_name = table.entity_name();
        let filename = format!("{}.rs", module_name);
        let output_dir = PathBuf::from(&self.global_config.output_dir).join(&self.package_config.mapper);
        let output_path = output_dir.join(&filename);

        let author = ctx.get("author").unwrap();
        let date = ctx.get("date").unwrap();
        let table_comment = &table.comment;

        let mut code = String::new();
        code.push_str(&format!("//! {}Mapper — {} 数据访问层\n", entity_name, table_comment));
        code.push_str(&format!("//!\n//! 生成时间: {}  作者: {}\n\n", date, author));
        code.push_str("use rbatis_plus_core::mapper::BaseMapper;\n\n");
        code.push_str(&format!("/// {} Mapper（对标 Java `BaseMapper<{}>`）\n", entity_name, entity_name));
        code.push_str(&format!("///\n/// 对应数据库表: `{}`\n", table.name));
        code.push_str(&format!("pub type {}Mapper = dyn BaseMapper<{}>;\n", entity_name, entity_name));

        self.write_file(&output_path, &code)?;
        Ok(output_path)
    }

    /// 生成 Service 文件。
    fn output_service(&self, table: &TableInfo, ctx: &HashMap<String, String>) -> Result<PathBuf, String> {
        let module_name = table.module_name();
        let entity_name = table.entity_name();
        let filename = format!("{}.rs", module_name);
        let output_dir = PathBuf::from(&self.global_config.output_dir).join(&self.package_config.service);
        let output_path = output_dir.join(&filename);

        let author = ctx.get("author").unwrap();
        let date = ctx.get("date").unwrap();

        let mut code = String::new();
        code.push_str(&format!("//! {}Service — {} 业务逻辑层\n", entity_name, table.comment));
        code.push_str(&format!("//!\n//! 生成时间: {}  作者: {}\n\n", date, author));
        code.push_str("use rbatis_plus_extension::service::IService;\n\n");
        code.push_str(&format!("/// {} Service（对标 Java `IService<{}>`）\n", entity_name, entity_name));
        code.push_str(&format!("///\n/// 对应数据库表: `{}`\n", table.name));
        code.push_str(&format!("pub type {}Service = dyn IService<{}>;\n", entity_name, entity_name));

        self.write_file(&output_path, &code)?;
        Ok(output_path)
    }

    /// 生成 Controller 文件。
    fn output_controller(&self, table: &TableInfo, ctx: &HashMap<String, String>) -> Result<PathBuf, String> {
        let module_name = table.module_name();
        let entity_name = table.entity_name();
        let filename = format!("{}.rs", module_name);
        let output_dir = PathBuf::from(&self.global_config.output_dir).join(&self.package_config.controller);
        let output_path = output_dir.join(&filename);

        let author = ctx.get("author").unwrap();
        let date = ctx.get("date").unwrap();

        let mut code = String::new();
        code.push_str(&format!("//! {}Controller — {} 接口层\n", entity_name, table.comment));
        code.push_str(&format!("//!\n//! 生成时间: {}  作者: {}\n\n", date, author));
        code.push_str(&format!("/// {} Controller（对标 Java `Controller<{}>`）\n", entity_name, entity_name));
        code.push_str(&format!("///\n/// 对应数据库表: `{}`\n", table.name));
        code.push_str(&format!("/// 通常使用 axum 框架实现，示例：\n"));
        code.push_str(&format!("/// ```ignore\n"));
        code.push_str(&format!("/// use axum::Router;\n"));
        code.push_str(&format!("/// use axum::routing::{{get, post, put, delete}};\n"));
        code.push_str(&format!("///\n"));
        code.push_str(&format!("/// pub fn {}_routes() -> Router {{\n", module_name));
        code.push_str(&format!("///     Router::new()\n"));
        code.push_str(&format!("///         .route(\"/\", get(list))\n"));
        code.push_str(&format!("///         .route(\"/:id\", get(get_by_id))\n"));
        code.push_str(&format!("///         .route(\"/\", post(create))\n"));
        code.push_str(&format!("///         .route(\"/:id\", put(update))\n"));
        code.push_str(&format!("///         .route(\"/:id\", delete(delete))\n"));
        code.push_str(&format!("/// }}\n"));
        code.push_str(&format!("/// ```\n"));

        self.write_file(&output_path, &code)?;
        Ok(output_path)
    }

    /// 写入文件（自动创建目录）。
    fn write_file(&self, path: &PathBuf, content: &str) -> Result<(), String> {
        if path.exists() && !self.global_config.file_override {
            log::info!("文件已存在，跳过: {:?}", path);
            return Ok(());
        }

        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
        }

        fs::write(path, content).map_err(|e| format!("写入文件失败: {}", e))?;
        log::info!("已生成: {:?}", path);
        Ok(())
    }
}
