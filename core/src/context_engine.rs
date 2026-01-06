use ignore::WalkBuilder;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use anyhow::Result;
use tracing::{warn};

pub struct ContextEngine;

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
                Ok(format!("<file path=\"{}\">
{}
</file>
", path_str, text))
            },
            Err(_) => Ok(String::new()), // Skip unreadable files
        }
    }

    fn is_binary(&self, path: &Path) -> Result<bool> {
        let mut file = File::open(path)?;
        let mut buffer = [0; 1024];
        // Read up to 1024 bytes
        let n = file.read(&mut buffer)?;
        
        // Simple check: look for null bytes
        for i in 0..n {
            if buffer[i] == 0 {
                return Ok(true);
            }
        }
        
        Ok(false)
    }
    
    // Helper for future use (as per spec)
    #[allow(dead_code)]
    fn estimate_tokens(&self, text: &str) -> usize {
        text.split_whitespace().count()
    }
}
