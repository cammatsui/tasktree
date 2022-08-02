use crate::tree::TaskTree;
use chrono::prelude::*;
use serde::{Deserialize, Serialize};
use std::{fs, io::Write};
use std::path::Path;


const FILE_EXTENSION: &str = ".json";
const TASKTREE_DIR: &str = ".tasktree/";
const ACTIVE_PROJ: &str = "active";
const PROJECTS_DIR: &str = "projects/";
pub const DATE_FORMAT: &str = "%m-%d-%Y %H:%M";


/// Struct representing a tasktree project.
#[derive(Debug, Serialize, Deserialize, PartialEq)]
pub struct Project {
    tasks: TaskTree,
    name: String,
    desc: String,
    created_timestamp: String,
    modified_timestamp: String,
}

impl Project {

    pub fn new(name: String, desc: String) -> Self {
        let cur_time: DateTime<Utc> = Utc::now();
        let created_timestamp = cur_time.format(DATE_FORMAT).to_string();
        let modified_timestamp = String::new();
        let mut proj = Project {
            tasks: TaskTree::new(),
            name,
            desc,
            created_timestamp,
            modified_timestamp,
        };
        proj.save().unwrap();
        proj
    }

    /// Get a list of tasktree project names.
    pub fn get_project_names() -> Result<Vec<String>, String> {
        let home: &str = env!("HOME");
        let project_path = format!("{}/{}{}", home, TASKTREE_DIR, PROJECTS_DIR);
        let err_msg = "Could not get project names.";
        match fs::create_dir_all(Path::new(&project_path)) {
            Ok(_) => (),
            _ => return Err(err_msg.to_string()), 
        }

        let mut proj_names = Vec::new();
        let files_result = fs::read_dir(project_path);
        let files = match files_result {
            Ok(result) => result,
            _ => return Err(err_msg.to_string()),
        };
        for file in files {
            let filename = String::from(
                format!("{}", file.unwrap().path().display())
                    .split("/")
                    .last()
                    .unwrap()
                    .replace(FILE_EXTENSION, ""),
            );
            proj_names.push(filename);
        }
        Ok(proj_names)
    }

    /// Check whether a tasktree project exists.
    pub fn exists(name: &str) -> Result<bool, String> {
        Ok(Self::get_project_names()?.contains(&name.to_string()))
    }

    /// Get the time at which this tasktree project was created.
    pub fn get_created_timestamp(&self) -> &str {
        &self.created_timestamp
    }

    /// Get the time at which this tasktree project was last modified.
    pub fn get_modified_timestamp(&self) -> &str {
        &&self.modified_timestamp
    }

    /// Get this project's name.
    pub fn get_name(&self) -> &str {
        &self.name
    }

    /// Get this project's description.
    pub fn get_desc(&self) -> &str {
        &self.desc
    }

    /// Get a reference to this project's tasktree.
    pub fn get_tree(&self) -> &TaskTree {
        &self.tasks
    }

    /// Get a mutable reference to this project's tasktree.
    pub fn get_tree_mut(&mut self) -> &mut TaskTree {
        &mut self.tasks
    }

    /// Save this project.
    pub fn save(&mut self) -> Result<(), String> {
        let cur_time: DateTime<Utc> = Utc::now();
        self.modified_timestamp = cur_time.format(DATE_FORMAT).to_string();

        let project_path = Self::get_project_path(&self.name);
        if Self::exists(&self.name)? {
            Self::remove(&self.name)?;
        }
        let err_msg = format!("Could not save project {}.", &self.name);

        let file_result = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(&project_path);


        let mut file = match file_result {
            Ok(result) => result,
            _ => return Err(err_msg),
        };
        let serialized = match serde_json::to_string(self) {
            Ok(result) => result,
            _ => return Err(err_msg),
        };
        match write!(file, "{}", serialized) {
            Err(_) => Err(err_msg),
            _ => Ok(()),
        }
    }

    /// Load a project.
    pub fn load(name: &str) -> Result<Self, String> {
        let project_path = Self::get_project_path(name);
        let err_msg = format!("Could not load project {}.", name);
        let read_str = match fs::read_to_string(project_path) {
            Ok(read) => read,
            _ => return Err(err_msg),
        };

        Ok(serde_json::from_str(read_str.trim()).unwrap())
    }

    /// Delete a project.
    pub fn remove(name: &str) -> Result<(), String> {
        let proj_path = Self::get_project_path(name);
        match fs::remove_file(proj_path) {
            Ok(_) => Ok(()),
            _ => Err(format!("Could not remove project {}.", name)),
        }
    }

    /// Set a project as the active project.
    pub fn set_active(project_name: &str) -> Result<(), String> {
        if !Self::exists(project_name)? {
            return Err(format!("There is no project named {}", project_name));
        }
        let mut file = fs::OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(Self::get_active_path())
            .unwrap();
        file.set_len(0).unwrap();
        file.write_all(project_name.as_bytes()).unwrap();
        Ok(())
    }

    /// Get the name of the active project.
    pub fn get_active() -> Option<String> {
        match &fs::read_to_string(Self::get_active_path()) {
            Ok(result) => Some(result.to_string()),
            Err(_) => None,
        }
    }

    fn get_active_path() -> String {
        let home: &str = env!("HOME");
        format!("{}/{}{}", home, TASKTREE_DIR, ACTIVE_PROJ)
    }

    fn get_project_path(project_name: &str) -> String {
        let home: &str = env!("HOME");
        let mut project_path = format!("{}/{}{}", home, TASKTREE_DIR, PROJECTS_DIR);
        fs::create_dir_all(&project_path).unwrap();
        project_path.push_str(&format!("/{}.json", project_name));
        project_path
    }

}



#[cfg(test)]
mod tests {
    use super::*;
    const TEST_PROJ: &str = "test_project";

    #[test]
    fn serialize_deserialize_test() {
        let tasktree = super::super::tree::tests::setup_tree();
        let mut project = Project {
            tasks: tasktree,
            name: TEST_PROJ.to_string(),
            desc: "desc".to_string(),
            created_timestamp: "asdf".to_string(),
            modified_timestamp: "sdfg".to_string(),
        };

        project.save().unwrap();
        assert!(Project::get_project_names().unwrap().contains(&TEST_PROJ.to_string()));
        assert!(Project::exists(TEST_PROJ).unwrap());

        Project::set_active(TEST_PROJ).unwrap();
        assert_eq!(Project::get_active().unwrap(), TEST_PROJ);

        let loaded_project = Project::load(TEST_PROJ).unwrap();
        assert_eq!(project, loaded_project);

        Project::remove(TEST_PROJ).unwrap();
        assert!(!Project::get_project_names().unwrap().contains(&TEST_PROJ.to_string()));
        assert!(!Project::exists(TEST_PROJ).unwrap());
    }
}
