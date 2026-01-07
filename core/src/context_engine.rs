use anyhow::Result;
use ignore::WalkBuilder;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use tracing::warn;

pub struct ContextEngine;

// ISPRAVAK 1: Implementacija `Default` traita
impl Default for ContextEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl ContextEngine {
    pub fn new() -> Self {
        Self
    }

    pub fn get_context(&self, root_path: &str) -> Result<String> {
        let mut context = String::new();

        let walker = WalkBuilder::new(root_path)
            .hidden(true) // Skip hidden files
            .git_ignore(true)
            .build();

        for result in walker {
            match result {
                Ok(entry) => {
                    let path = entry.path();
                    if path.is_file() {
                        if let Ok(content) = self.process_file(path) {
                            context.push_str(&content);
                        }
                    }
                }
                Err(err) => warn!("Error walking directory: {}", err),
            }
        }

        Ok(context)
    }

    fn process_file(&self, path: &Path) -> Result<String> {
        // 1. Binary Check
        if self.is_binary(path)? {
            return Ok(String::new());
        }

        // 2. Read Content
        let content = std::fs::read_to_string(path);
        match content {
            Ok(text) => {
                // 3. Format as XML
                let path_str = path.to_string_lossy();
                Ok(format!(
                    "<file path=\"{}\">
{}
</file>
",
                    path_str, text
                ))
            }
            Err(_) => Ok(String::new()), // Skip unreadable files
        }
    }

    fn is_binary(&self, path: &Path) -> Result<bool> {
        let mut file = File::open(path)?;
        let mut buffer = [0; 1024];
        let n = file.read(&mut buffer)?;

        // ISPRAVAK 2: Korištenje iteratora umjesto `for i in 0..n`
        Ok(buffer.iter().take(n).any(|&byte| byte == 0))
    }

    // Helper for future use (as per spec)
    #[allow(dead_code)]
    fn estimate_tokens(&self, text: &str) -> usize {
        text.split_whitespace().count()
    }
}
