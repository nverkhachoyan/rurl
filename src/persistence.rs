use rusqlite::{params, Connection, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug)]
pub enum ProjectUpdate {
    AddRequest(RequestData),
    UpdateRequest(usize, RequestData),
    DeleteRequest(usize),
    UpdateName(String),
    AddEnvironment(Environment),
    UpdateEnvironment(usize, Environment),
    DeleteEnvironment(usize),
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ProjectData {
    pub name: String,
    pub id: String,
    pub requests: Vec<RequestData>,
    pub environments: Vec<Environment>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct Environment {
    pub name: String,
    pub variables: HashMap<String, String>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct RequestData {
    pub name: String,
    pub method: Option<String>,
    pub url: Option<String>,
    pub headers: Option<Vec<(String, String)>>,
    pub body: Option<String>,
    pub query_params: Option<Vec<(String, String)>>,
    pub path_params: Option<Vec<(String, String)>>,
    pub auth: Option<AuthData>,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub enum AuthData {
    None,
    Basic {
        username: String,
        password: String,
    },
    Bearer {
        token: String,
    },
    ApiKey {
        key: String,
        value: String,
        in_header: bool,
    },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
pub struct ResponseData {
    pub request_id: i64,
    pub status_code: Option<i32>,
    pub response_body: Option<String>,
    pub response_headers: Option<Vec<(String, String)>>,
    pub response_time: i64,
    pub timestamp: i64,
}

impl ProjectData {
    pub fn new(name: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Self {
            name,
            id: uuid::Uuid::new_v4().to_string(),
            requests: Vec::new(),
            environments: Vec::new(),
            created_at: now,
            updated_at: now,
        }
    }

    pub fn apply_update(&mut self, update: ProjectUpdate) {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;
        self.updated_at = now;

        match update {
            ProjectUpdate::AddRequest(request) => {
                self.requests.push(request);
            }
            ProjectUpdate::UpdateRequest(index, request) => {
                if index < self.requests.len() {
                    self.requests[index] = request;
                }
            }
            ProjectUpdate::DeleteRequest(index) => {
                if index < self.requests.len() {
                    self.requests.remove(index);
                }
            }
            ProjectUpdate::UpdateName(new_name) => {
                self.name = new_name;
            }
            ProjectUpdate::AddEnvironment(env) => {
                self.environments.push(env);
            }
            ProjectUpdate::UpdateEnvironment(index, env) => {
                if index < self.environments.len() {
                    self.environments[index] = env;
                }
            }
            ProjectUpdate::DeleteEnvironment(index) => {
                if index < self.environments.len() {
                    self.environments.remove(index);
                }
            }
        }
    }
}

impl RequestData {
    pub fn new(name: String) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64;

        Self {
            name,
            method: None,
            url: None,
            headers: None,
            body: None,
            query_params: None,
            path_params: None,
            auth: Some(AuthData::None),
            created_at: now,
            updated_at: now,
        }
    }
}

pub struct Storage {
    conn: Connection,
}

impl Storage {
    pub fn new() -> Self {
        let mut path = dirs::data_local_dir().unwrap_or_else(|| PathBuf::from("."));
        path.push("rurl");
        std::fs::create_dir_all(&path).unwrap_or_else(|e| {
            eprintln!("Failed to create storage directory: {}", e);
        });
        path.push("rurl.db");

        let conn = Connection::open(&path).expect("Failed to open database");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS projects (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL
            )",
            [],
        )
        .expect("Failed to create projects table");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS environments (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id TEXT NOT NULL,
                name TEXT NOT NULL,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                FOREIGN KEY(project_id) REFERENCES projects(id)
            )",
            [],
        )
        .expect("Failed to create environments table");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS environment_variables (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                environment_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                value TEXT NOT NULL,
                FOREIGN KEY(environment_id) REFERENCES environments(id)
            )",
            [],
        )
        .expect("Failed to create environment_variables table");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS requests (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                project_id TEXT NOT NULL,
                name TEXT NOT NULL,
                method TEXT,
                url TEXT,
                body TEXT,
                query_params TEXT,
                path_params TEXT,
                auth_data TEXT,
                created_at INTEGER NOT NULL,
                updated_at INTEGER NOT NULL,
                FOREIGN KEY(project_id) REFERENCES projects(id)
            )",
            [],
        )
        .expect("Failed to create requests table");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS headers (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                request_id INTEGER NOT NULL,
                name TEXT NOT NULL,
                value TEXT NOT NULL,
                FOREIGN KEY(request_id) REFERENCES requests(id)
            )",
            [],
        )
        .expect("Failed to create headers table");

        conn.execute(
            "CREATE TABLE IF NOT EXISTS request_history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                request_id INTEGER NOT NULL,
                status_code INTEGER,
                response_body TEXT,
                response_headers TEXT,
                response_time INTEGER NOT NULL,
                timestamp INTEGER NOT NULL,
                FOREIGN KEY(request_id) REFERENCES requests(id)
            )",
            [],
        )
        .expect("Failed to create request_history table");

        Self { conn }
    }

    pub fn save_project(
        &mut self,
        project: &ProjectData,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let tx = self.conn.transaction()?;

        tx.execute(
            "INSERT OR REPLACE INTO projects (id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
            params![project.id, project.name, project.created_at, project.updated_at],
        )?;

        tx.execute(
            "DELETE FROM environment_variables WHERE environment_id IN (
                SELECT id FROM environments WHERE project_id = ?1
            )",
            params![project.id],
        )?;
        tx.execute(
            "DELETE FROM environments WHERE project_id = ?1",
            params![project.id],
        )?;

        for env in &project.environments {
            let env_id = {
                tx.execute(
                    "INSERT INTO environments (project_id, name, created_at, updated_at) VALUES (?1, ?2, ?3, ?4)",
                    params![project.id, env.name, project.created_at, project.updated_at],
                )?;
                tx.last_insert_rowid()
            };

            for (name, value) in &env.variables {
                tx.execute(
                    "INSERT INTO environment_variables (environment_id, name, value) VALUES (?1, ?2, ?3)",
                    params![env_id, name, value],
                )?;
            }
        }

        tx.execute(
            "DELETE FROM headers WHERE request_id IN (
                SELECT id FROM requests WHERE project_id = ?1
            )",
            params![project.id],
        )?;
        tx.execute(
            "DELETE FROM requests WHERE project_id = ?1",
            params![project.id],
        )?;

        for request in &project.requests {
            let request_id = {
                tx.execute(
                    "INSERT INTO requests (
                        project_id, name, method, url, body, 
                        query_params, path_params, auth_data,
                        created_at, updated_at
                    ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
                    params![
                        project.id,
                        request.name,
                        request.method,
                        request.url,
                        request.body,
                        serde_json::to_string(&request.query_params)?,
                        serde_json::to_string(&request.path_params)?,
                        serde_json::to_string(&request.auth)?,
                        request.created_at,
                        request.updated_at,
                    ],
                )?;
                tx.last_insert_rowid()
            };

            if let Some(headers) = &request.headers {
                for (name, value) in headers {
                    tx.execute(
                        "INSERT INTO headers (request_id, name, value) VALUES (?1, ?2, ?3)",
                        params![request_id, name, value],
                    )?;
                }
            }
        }

        tx.commit()?;
        Ok(())
    }

    pub fn load_project(
        &mut self,
        id: &str,
    ) -> Result<Option<ProjectData>, Box<dyn std::error::Error>> {
        let mut stmt = self
            .conn
            .prepare("SELECT name, created_at, updated_at FROM projects WHERE id = ?1")?;

        let project = stmt.query_row(params![id], |row| {
            Ok(ProjectData {
                id: id.to_string(),
                name: row.get(0)?,
                requests: Vec::new(),
                environments: Vec::new(),
                created_at: row.get(1)?,
                updated_at: row.get(2)?,
            })
        });

        let mut project = match project {
            Ok(p) => p,
            Err(rusqlite::Error::QueryReturnedNoRows) => return Ok(None),
            Err(e) => return Err(Box::new(e)),
        };

        let mut stmt = self.conn.prepare(
            "SELECT id, name, created_at, updated_at FROM environments WHERE project_id = ?1",
        )?;
        let env_rows = stmt.query_map(params![id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                Environment {
                    name: row.get(1)?,
                    variables: HashMap::new(),
                },
            ))
        })?;

        let mut environments = Vec::new();
        for env_row in env_rows {
            let (env_id, mut env) = env_row?;

            let mut stmt = self.conn.prepare(
                "SELECT name, value FROM environment_variables WHERE environment_id = ?1",
            )?;
            let vars: Vec<(String, String)> = stmt
                .query_map(params![env_id], |row| Ok((row.get(0)?, row.get(1)?)))?
                .collect::<Result<Vec<_>, _>>()?;

            env.variables = vars.into_iter().collect();
            environments.push(env);
        }
        project.environments = environments;

        let mut stmt = self.conn.prepare(
            "SELECT id, name, method, url, body, query_params, path_params, auth_data, created_at, updated_at 
             FROM requests WHERE project_id = ?1",
        )?;
        let request_rows = stmt.query_map(params![id], |row| {
            Ok((
                row.get::<_, i64>(0)?,
                RequestData {
                    name: row.get(1)?,
                    method: row.get(2)?,
                    url: row.get(3)?,
                    headers: Some(Vec::new()),
                    body: row.get(4)?,
                    query_params: serde_json::from_str(&row.get::<_, String>(5)?).ok(),
                    path_params: serde_json::from_str(&row.get::<_, String>(6)?).ok(),
                    auth: serde_json::from_str(&row.get::<_, String>(7)?).ok(),
                    created_at: row.get(8)?,
                    updated_at: row.get(9)?,
                },
            ))
        })?;

        let mut requests = Vec::new();
        for request_row in request_rows {
            let (request_id, mut request) = request_row?;

            let mut stmt = self
                .conn
                .prepare("SELECT name, value FROM headers WHERE request_id = ?1")?;
            let headers: Vec<(String, String)> = stmt
                .query_map(params![request_id], |row| Ok((row.get(0)?, row.get(1)?)))?
                .collect::<Result<Vec<_>, _>>()?;

            request.headers = Some(headers);
            requests.push(request);
        }

        project.requests = requests;
        Ok(Some(project))
    }

    pub fn list_projects(&mut self) -> Vec<ProjectData> {
        let mut stmt = self
            .conn
            .prepare("SELECT id, name FROM projects ORDER BY id")
            .expect("Failed to prepare statement");

        let projects = stmt
            .query_map([], |row| {
                Ok(ProjectData {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    requests: Vec::new(),
                    environments: Vec::new(),
                    created_at: 0,
                    updated_at: 0,
                })
            })
            .expect("Failed to execute query");

        projects.filter_map(Result::ok).collect()
    }

    pub fn delete_project(&mut self, id: &str) -> Result<(), Box<dyn std::error::Error>> {
        let tx = self.conn.transaction()?;

        tx.execute(
            "DELETE FROM headers WHERE request_id IN (
                SELECT id FROM requests WHERE project_id = ?1
            )",
            params![id],
        )?;
        tx.execute("DELETE FROM requests WHERE project_id = ?1", params![id])?;
        tx.execute("DELETE FROM projects WHERE id = ?1", params![id])?;
        tx.commit()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_crud() {
        let mut storage = Storage::new();

        let project = ProjectData {
            name: "Test Project".to_string(),
            id: "test-123".to_string(),
            requests: vec![RequestData {
                name: "".to_string(),
                method: Some("GET".to_string()),
                url: Some("https://api.example.com".to_string()),
                headers: Some(vec![(
                    "Content-Type".to_string(),
                    "application/json".to_string(),
                )]),
                body: None,
                query_params: None,
                path_params: None,
                auth: None,
                created_at: 0,
                updated_at: 0,
            }],
            environments: Vec::new(),
            created_at: 0,
            updated_at: 0,
        };

        storage.save_project(&project).unwrap();

        let loaded = storage.load_project(&project.id).unwrap().unwrap();
        assert_eq!(project.name, loaded.name);
        assert_eq!(project.id, loaded.id);
        assert_eq!(project.requests.len(), loaded.requests.len());

        let projects = storage.list_projects();
        assert!(projects.iter().any(|p| p.id == project.id));

        storage.delete_project(&project.id).unwrap();
        assert!(storage.load_project(&project.id).unwrap().is_none());
    }
}
