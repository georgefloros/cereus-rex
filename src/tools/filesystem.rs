// src/tools/filesystem.rs
use serde::{Deserialize, Serialize};
use std::path::Path;
use tokio::fs;

#[derive(Serialize, Deserialize, Debug)]
pub struct FileOperationRequest {
    pub operation: String, // "read", "write", "list", "create", "delete"
    pub path: String,
    pub content: Option<String>,
    pub recursive: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct FileOperationResponse {
    pub success: bool,
    pub message: String,
    pub data: Option<serde_json::Value>,
}

pub struct FilesystemTool;

impl FilesystemTool {
    pub async fn execute(&self, request: FileOperationRequest) -> Result<FileOperationResponse, Box<dyn std::error::Error>> {
        match request.operation.as_str() {
            "read" => self.read_file(&request.path).await,
            "write" => {
                if let Some(content) = request.content {
                    self.write_file(&request.path, &content).await
                } else {
                    Ok(FileOperationResponse {
                        success: false,
                        message: "Content required for write operation".to_string(),
                        data: None,
                    })
                }
            },
            "list" => {
                let recursive = request.recursive.unwrap_or(false);
                self.list_directory(&request.path, recursive).await
            },
            "create" => self.create_file(&request.path).await,
            "delete" => self.delete_file(&request.path).await,
            _ => Ok(FileOperationResponse {
                success: false,
                message: format!("Unsupported operation: {}", request.operation),
                data: None,
            }),
        }
    }

    async fn read_file(&self, path: &str) -> Result<FileOperationResponse, Box<dyn std::error::Error>> {
        match fs::read_to_string(path).await {
            Ok(content) => Ok(FileOperationResponse {
                success: true,
                message: "File read successfully".to_string(),
                data: Some(serde_json::json!({
                    "content": content,
                    "path": path
                })),
            }),
            Err(e) => Ok(FileOperationResponse {
                success: false,
                message: format!("Failed to read file: {}", e),
                data: None,
            }),
        }
    }

    async fn write_file(&self, path: &str, content: &str) -> Result<FileOperationResponse, Box<dyn std::error::Error>> {
        match fs::write(path, content).await {
            Ok(_) => Ok(FileOperationResponse {
                success: true,
                message: "File written successfully".to_string(),
                data: Some(serde_json::json!({
                    "path": path
                })),
            }),
            Err(e) => Ok(FileOperationResponse {
                success: false,
                message: format!("Failed to write file: {}", e),
                data: None,
            }),
        }
    }

    async fn list_directory(&self, path: &str, recursive: bool) -> Result<FileOperationResponse, Box<dyn std::error::Error>> {
        if recursive {
            self.list_directory_recursive(path).await
        } else {
            self.list_directory_simple(path).await
        }
    }

    async fn list_directory_simple(&self, path: &str) -> Result<FileOperationResponse, Box<dyn std::error::Error>> {
        let mut entries = tokio::fs::read_dir(path).await?;
        let mut files = Vec::new();

        while let Some(entry) = entries.next_entry().await? {
            let file_type = entry.file_type().await?;
            let entry_info = serde_json::json!({
                "name": entry.file_name().to_string_lossy().to_string(),
                "path": entry.path().to_string_lossy().to_string(),
                "is_file": file_type.is_file(),
                "is_directory": file_type.is_dir(),
            });
            files.push(entry_info);
        }

        Ok(FileOperationResponse {
            success: true,
            message: "Directory listed successfully".to_string(),
            data: Some(serde_json::json!(files)),
        })
    }

    async fn list_directory_recursive(&self, path: &str) -> Result<FileOperationResponse, Box<dyn std::error::Error>> {
        let mut files = Vec::new();
        self.collect_files_recursive(Path::new(path), &mut files).await?;

        Ok(FileOperationResponse {
            success: true,
            message: "Directory listed recursively successfully".to_string(),
            data: Some(serde_json::json!(files)),
        })
    }

    async fn collect_files_recursive(&self, dir: &Path, files: &mut Vec<serde_json::Value>) -> Result<(), Box<dyn std::error::Error>> {
        let mut dir_queue = Vec::new();
        dir_queue.push(dir.to_path_buf());

        while let Some(current_dir) = dir_queue.pop() {
            let mut entries = tokio::fs::read_dir(&current_dir).await?;

            while let Some(entry) = entries.next_entry().await? {
                let path = entry.path();
                let file_type = entry.file_type().await?;

                let entry_info = serde_json::json!({
                    "name": entry.file_name().to_string_lossy().to_string(),
                    "path": path.to_string_lossy().to_string(),
                    "is_file": file_type.is_file(),
                    "is_directory": file_type.is_dir(),
                });

                files.push(entry_info);

                if file_type.is_dir() {
                    dir_queue.push(path);
                }
            }
        }

        Ok(())
    }

    async fn create_file(&self, path: &str) -> Result<FileOperationResponse, Box<dyn std::error::Error>> {
        match fs::File::create(path).await {
            Ok(_) => Ok(FileOperationResponse {
                success: true,
                message: "File created successfully".to_string(),
                data: Some(serde_json::json!({
                    "path": path
                })),
            }),
            Err(e) => Ok(FileOperationResponse {
                success: false,
                message: format!("Failed to create file: {}", e),
                data: None,
            }),
        }
    }

    async fn delete_file(&self, path: &str) -> Result<FileOperationResponse, Box<dyn std::error::Error>> {
        let path_obj = Path::new(path);
        
        if path_obj.is_file() {
            match fs::remove_file(path).await {
                Ok(_) => Ok(FileOperationResponse {
                    success: true,
                    message: "File deleted successfully".to_string(),
                    data: Some(serde_json::json!({
                        "path": path
                    })),
                }),
                Err(e) => Ok(FileOperationResponse {
                    success: false,
                    message: format!("Failed to delete file: {}", e),
                    data: None,
                }),
            }
        } else if path_obj.is_dir() {
            match fs::remove_dir_all(path).await {
                Ok(_) => Ok(FileOperationResponse {
                    success: true,
                    message: "Directory deleted successfully".to_string(),
                    data: Some(serde_json::json!({
                        "path": path
                    })),
                }),
                Err(e) => Ok(FileOperationResponse {
                    success: false,
                    message: format!("Failed to delete directory: {}", e),
                    data: None,
                }),
            }
        } else {
            Ok(FileOperationResponse {
                success: false,
                message: format!("Path does not exist: {}", path),
                data: None,
            })
        }
    }
}