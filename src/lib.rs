//! jrust 的 OpenIDEA 插件。
//!
//! - 语言注册（`languages/jrust.toml`）：`.jrust` 文件用 Java 模式高亮，并声明 DAP 调试器。
//! - 新建项目：`jrust: 新建项目…` 命令（工具菜单 / 查找操作）调用 `jrust new` 生成骨架并打开。
//! - 运行配置：项目里每个 `.jrust` 文件都生成一条 `jrust run <文件>`。
//! - 调试：配置的 `debugger` 留空，IDE 从 args 里的 `.jrust` 文件推断出 jrust 语言，
//!   再启动该语言的 `[debugger]`（`jrust dap`，stdio）；断点落在 `.jrust` 源码行上。

use std::collections::BTreeSet;

use openidea_plugin::{
    Error, Invocation, LaunchConfigurations, Plugin, PluginLaunchConfiguration, PluginRunConfiguration,
    host, register_plugin,
};

#[derive(Default)]
struct JrustPlugin;

impl Plugin for JrustPlugin {
    fn run_command(
        &mut self,
        id: &str,
        _ctx: &Invocation,
        _arg: Option<&str>,
    ) -> Result<Option<String>, Error> {
        match id {
            "new-project" => new_project(),
            _ => Err(Error::Invalid(format!("jrust 插件没有命令: {id}"))),
        }
    }
}

/// “新建项目”命令：选目录、输名称，交给 `jrust new` 生成骨架，再用 IDE 正常流程打开。
/// 项目生成器目录内置在 IDE 里，插件只能提供这个等效入口。
fn new_project() -> Result<Option<String>, Error> {
    let Some(parent) = host::pick_directory("选择 jrust 项目所在目录", None)? else {
        return Ok(None);
    };
    let Some(name) = host::input("jrust 项目名称", Some("hello-jrust"))? else {
        return Ok(None);
    };
    let name = name.trim();
    if name.is_empty() || name == "." || name == ".." || name.contains(['/', '\\']) {
        return Err(Error::Invalid(format!("项目名称不合法: {name}")));
    }
    let target = std::path::Path::new(&parent).join(name);
    let target = target.to_string_lossy().into_owned();

    let out = host::run_process("jrust", &["new".to_string(), target.clone()], None)?;
    if out.status != Some(0) {
        let message = if out.stderr.trim().is_empty() { out.stdout } else { out.stderr };
        return Err(Error::Failed(message.trim().to_string()));
    }
    host::open_project(&target, true)?;
    Ok(Some(format!("已创建 jrust 项目 {target}")))
}

impl LaunchConfigurations for JrustPlugin {
    fn launch_configurations(&mut self, project_root: &str) -> Vec<PluginLaunchConfiguration> {
        let root = normalize(project_root);
        let Some(project) = host::projects()
            .ok()
            .and_then(|projects| {
                projects
                    .into_iter()
                    .find(|p| p.root().ok().map(|r| normalize(&r)).as_deref() == Some(root.as_str()))
            })
        else {
            return Vec::new();
        };

        let mut files = project.find_files("**/*.jrust", 500).unwrap_or_default();
        files.sort();
        files.dedup();

        let mut used = BTreeSet::new();
        files
            .into_iter()
            .filter_map(|path| {
                let rel = relative(&path, &root)?;
                let id = unique_id(&rel, &mut used);
                Some(PluginLaunchConfiguration {
                    config: PluginRunConfiguration {
                        id,
                        name: format!("jrust: {rel}"),
                        program: "jrust".into(),
                        args: vec!["run".into(), path],
                        cwd: Some(root.clone()),
                        ..Default::default()
                    },
                    env: Vec::new(),
                    // None：Debug 从 args 里的 `.jrust` 源文件推断语言，再用该语言的
                    // `[debugger]`（jrust → `jrust dap`）。
                    debugger: None,
                })
            })
            .collect()
    }
}

/// 统一分隔符并去掉末尾的 `/`，便于比较项目根。
fn normalize(path: &str) -> String {
    path.replace('\\', "/").trim_end_matches('/').to_string()
}

/// 项目根相对路径（`/` 分隔）；不在根下时返回 None。
fn relative(path: &str, root: &str) -> Option<String> {
    let p = normalize(path);
    let rel = p.strip_prefix(root)?.trim_start_matches('/').to_string();
    if rel.is_empty() { None } else { Some(rel) }
}

/// 运行配置 id 只能匹配 `[a-z0-9][a-z0-9._-]*`（≤64 字符）。
fn config_id(rel: &str) -> String {
    let mut id = String::with_capacity(rel.len());
    for ch in rel.chars() {
        if ch.is_ascii_alphanumeric() {
            id.push(ch.to_ascii_lowercase());
        } else {
            id.push('.');
        }
    }
    let id: String = id.trim_matches('.').chars().take(64).collect();
    if id.is_empty() { "main".into() } else { id }
}

fn unique_id(rel: &str, used: &mut BTreeSet<String>) -> String {
    let base = config_id(rel);
    let mut id = base.clone();
    let mut n = 2;
    while !used.insert(id.clone()) {
        let stem: String = base.chars().take(60).collect();
        id = format!("{stem}-{n}");
        n += 1;
    }
    id
}

register_plugin!(JrustPlugin, launch_configurations);
