use std::{
    fs,
    path::{Path, PathBuf},
};

// ── trait ─────────────────────────────────────────────────────────────────

/// 类文件搜索路径。
///
/// 实现者只需提供 `read_class`，根据全限定类名（如 `java/lang/Object`）
/// 返回对应的 `.class` 文件字节。
pub trait ClassPath: Send + Sync {
    /// 读取指定类的字节码。未找到时返回 `None`。
    fn read_class(&self, name: &str) -> Option<Vec<u8>>;
}

// ── 目录类路径 ───────────────────────────────────────────────────────────

/// 普通目录类路径，将 `java/lang/Object` 映射为 `$base/java/lang/Object.class`。
pub struct DirClassPath {
    base: PathBuf,
}

impl DirClassPath {
    pub fn new(base: impl Into<PathBuf>) -> Self {
        Self { base: base.into() }
    }
}

impl ClassPath for DirClassPath {
    fn read_class(&self, name: &str) -> Option<Vec<u8>> {
        let path = self.base.join(format!("{}.class", name));
        fs::read(&path).ok()
    }
}

// ── 模块路径（JDK 9+） ───────────────────────────────────────────────────

/// JDK 模块目录的类路径。
///
/// 当前实现与 [`DirClassPath`] 完全一致——拼接路径读取 `.class` 文件。
/// 保留为独立类型是为了后续扩展：
///
/// - 读取模块根目录下的 `module.toml`，解析 `[exports]`、`[[depends]]`
///   等声明；
/// - `read_class` 时根据 exports 规则检查调用方模块是否有权访问该类；
/// - 自动加载 `module.toml` 中声明的依赖模块（`[[depends]]`）。
///
/// 在模块系统未实现前，`ModulePath` 等价于 `DirClassPath`。
pub struct ModulePath {
    module_base: PathBuf,
}

impl ModulePath {
    pub fn new(module_base: impl Into<PathBuf>) -> Self {
        Self {
            module_base: module_base.into(),
        }
    }

    /// 从 JDK 根目录 + 模块名构造。
    ///
    /// 例如 `from_jdk_root("/path/to/jdk", "java.base")`。
    pub fn from_jdk_root(jdk_root: &Path, module_name: &str) -> Self {
        Self {
            module_base: jdk_root.join(module_name),
        }
    }
}

impl ClassPath for ModulePath {
    fn read_class(&self, name: &str) -> Option<Vec<u8>> {
        let path = self.module_base.join(format!("{}.class", name));
        fs::read(&path).ok()
    }
}

// ── 组合路径 ─────────────────────────────────────────────────────────────

/// 多条搜索路径的组合，按顺序查找，先到先得。
pub struct CompositeClassPath {
    entries: Vec<Box<dyn ClassPath>>,
}

impl CompositeClassPath {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    /// 追加搜索路径。后加入的优先级低。
    pub fn push(&mut self, cp: Box<dyn ClassPath>) {
        self.entries.push(cp);
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

impl Default for CompositeClassPath {
    fn default() -> Self {
        Self::new()
    }
}

impl ClassPath for CompositeClassPath {
    fn read_class(&self, name: &str) -> Option<Vec<u8>> {
        self.entries.iter().find_map(|cp| cp.read_class(name))
    }
}

// ── 测试用内存路径 ───────────────────────────────────────────────────────

/// 仅用于单元测试。直接注入类名 → 字节的映射。
#[cfg(test)]
pub struct MemClassPath {
    classes: std::collections::HashMap<String, Vec<u8>>,
}

#[cfg(test)]
impl MemClassPath {
    pub fn new(classes: std::collections::HashMap<String, Vec<u8>>) -> Self {
        Self { classes }
    }
}

#[cfg(test)]
impl ClassPath for MemClassPath {
    fn read_class(&self, name: &str) -> Option<Vec<u8>> {
        self.classes.get(name).cloned()
    }
}

// ── 测试 ─────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_composite_fallback() {
        let mut mem = std::collections::HashMap::new();
        mem.insert("foo/Bar".to_string(), b"bar".to_vec());

        let mut cp = CompositeClassPath::new();
        cp.push(Box::new(MemClassPath::new(mem)));
        cp.push(Box::new(
            MemClassPath::new(std::collections::HashMap::new()),
        ));

        assert_eq!(cp.read_class("foo/Bar"), Some(b"bar".to_vec()));
        assert_eq!(cp.read_class("not/Found"), None);
    }

    #[test]
    fn test_composite_priority() {
        let mut first = std::collections::HashMap::new();
        first.insert("pkg/A".to_string(), b"first".to_vec());

        let mut second = std::collections::HashMap::new();
        second.insert("pkg/A".to_string(), b"second".to_vec());

        let mut cp = CompositeClassPath::new();
        cp.push(Box::new(MemClassPath::new(first)));
        cp.push(Box::new(MemClassPath::new(second)));

        // 先到先得
        assert_eq!(cp.read_class("pkg/A"), Some(b"first".to_vec()));
    }

    #[test]
    fn test_dir_class_path() {
        let dir = std::env::temp_dir().join("myjvm_test_dir_cp");
        let cls_dir = dir.join("pkg");
        fs::create_dir_all(&cls_dir).unwrap();
        fs::write(cls_dir.join("Test.class"), b"hello").unwrap();

        let cp = DirClassPath::new(&dir);
        assert_eq!(cp.read_class("pkg/Test"), Some(b"hello".to_vec()));
        assert_eq!(cp.read_class("pkg/NotFound"), None);

        fs::remove_dir_all(&dir).unwrap();
    }
}
