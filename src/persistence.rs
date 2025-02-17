use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ProjectData {
    pub name: String,
    pub id: String,
    pub requests: Vec<RequestData>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct RequestData {
    pub method: String,
    pub url: String,
    pub headers: Vec<(String, String)>,
    pub body: Option<String>,
}

pub struct Storage {
    storage_path: PathBuf,
}

impl Storage {
    pub fn new() -> Self {
        let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("rurl");
        path.push("projects");

        fs::create_dir_all(&path).unwrap_or_else(|e| {
            eprintln!("failed to create storage directory: {}", e);
        });

        Self { storage_path: path }
    }

    pub fn save_project(&self, project: &ProjectData) -> Result<(), Box<dyn std::error::Error>> {
        let mut file_path = self.storage_path.clone();
        file_path.push(format!("{}.json", project.id));

        let json = serde_json::to_string_pretty(project)?;
        fs::write(file_path, json)?;
        Ok(())
    }

    pub fn load_project(
        &self,
        id: &str,
    ) -> Result<Option<ProjectData>, Box<dyn std::error::Error>> {
        let mut file_path = self.storage_path.clone();
        file_path.push(format!("{}.json", id));

        let json = fs::read_to_string(file_path)?;

        match json.is_empty() {
            true => Ok(None),
            false => {
                let project: ProjectData = serde_json::from_str(&json)?;
                Ok(Some(project))
            }
        }
    }

    pub fn list_projects(&self) -> Vec<ProjectData> {
        fs::read_dir(&self.storage_path)
            .map(|entries| {
                entries
                    .filter_map(|entry| {
                        entry.ok().and_then(|e| {
                            fs::read_to_string(e.path()).ok().and_then(|contents| {
                                serde_json::from_str::<ProjectData>(&contents)
                                    .ok()
                                    .map(|project| project)
                            })
                        })
                    })
                    .collect()
            })
            .unwrap_or_else(|_| vec![])
    }

    pub fn delete_project(&self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let mut file_path = self.storage_path.clone();
        file_path.push(format!("{}.json", id));
        fs::remove_file(file_path)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_serialization() {
        let project = ProjectData {
            name: "Test Project".to_string(),
            id: "test-123".to_string(),
            requests: vec![RequestData {
                method: "GET".to_string(),
                url: "https://api.example.com".to_string(),
                headers: vec![("Content-Type".to_string(), "application/json".to_string())],
                body: None,
            }],
        };

        let json = serde_json::to_string_pretty(&project).unwrap();
        let deserialized: ProjectData = serde_json::from_str(&json).unwrap();
        assert_eq!(project.name, deserialized.name);
        assert_eq!(project.id, deserialized.id);
    }
}
