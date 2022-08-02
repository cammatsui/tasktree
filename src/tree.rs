use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use chrono::prelude::*;
use crate::project::DATE_FORMAT;
use crate::command::{ bold_text, bold_tid, underline_text };


pub type TID = u16;


/// A struct representing a project's task dependency graph (tasktree).
#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct TaskTree {
    id_counter: TID,
    tasks: HashMap<TID, Box<Task>> ,
    children: HashMap<TID, Vec<TID>>,
    parents: HashMap<TID, Vec<TID>>,
}

impl TaskTree {

    pub fn new() -> TaskTree {
        TaskTree {
            id_counter: 1,
            tasks: HashMap::new(),
            children: HashMap::new(),
            parents: HashMap::new(),
        }
    }

    /// Creates a new task with the given description and adds it to the tree.
    pub fn add_task(&mut self, name: String, desc: Option<String>) -> TID {
        let id = self.id_counter;
        self.tasks.insert(id, Box::new(Task::new(id, name, desc)));
        self.children.insert(id, Vec::new());
        self.parents.insert(id, Vec::new());
        self.id_counter += 1;
        id
    }

    /// Gets the representation of the task if exists.
    pub fn get_task_repr(&self, task_id: &TID) -> Option<String> {
        match self.tasks.get(task_id) {
            None => None,
            Some(task) => Some(task.get_repr().to_string()),
        }
    }

    /// Removes the task with the given TID from the tree. Also removes all of its dependencies and 
    /// any dependencies on it. If the task does not exist, warn user.
    pub fn remove_task(&mut self, task_id: &TID) -> Result<(), String> {
        self.check_task_exists(task_id)?;
        if self.tasks.contains_key(task_id) {
            // remove from this task's parents' children
            let this_parents = self.parents.get(task_id).unwrap();
            for parent_id in this_parents.iter() {
                let parent_children = self.children.get_mut(parent_id).unwrap();
                parent_children.retain(|id| id != task_id);
            }

            // remove from this task's childrens' parents
            let this_children = self.children.get(task_id).unwrap();
            for child_id in this_children.iter() {
                let child_parents = self.parents.get_mut(child_id).unwrap();
                child_parents.retain(|id| id != task_id);
            }
            // TODO: add orphan check

            self.tasks.remove(task_id);
            Ok(())
        } else {
            Err(format!("No task with id {} in active project.", bold_tid(*task_id)))
        }
    }

    /// View project tasks by the status. If no status flag is provided, shows all available tasks.
    /// If the status_flag is "all", view all tasks. If the provided status_flag is invalid,
    /// informs user.
    pub fn view_tasks(&self, status_flag: Option<String>) -> Result<Vec<&Task>, String> {
        match status_flag {
            None => return Ok(self.get_available_tasks()),
            _ => ()
        };

        let flag = status_flag.unwrap();
        let parseable_status = match TaskStatus::from_status_flag(&flag) {
            Ok(_) => true,
            _ => false,
        };

        if flag != "all" && !parseable_status {
            return Err(format!("Invalid status flag {}.", bold_text(&flag)));
        }

        let parsed_status = TaskStatus::from_status_flag(&flag);

        let mut tasks = Vec::new();
        for task in self.tasks.values() {
            if flag == "all" || task.status == *parsed_status.as_ref().unwrap() {
                tasks.push(&**task);
            }
        }
        tasks.sort_by_key(|task| task.get_id());
        Ok(tasks)
    }

    /// Search this tree's tasks with the given query and optional status flag. If no status
    /// flag is provided, searches all tasks. Returns a vector of reprs for the matching tasks.
    pub fn search_tasks(
        &self,
        query: &str,
        opt_status_flag: Option<String>
    ) -> Result<Vec<String>, String> {
        let opt_status = TaskStatus::from_opt_status_flag(opt_status_flag)?;
        let tasks_iter = self.tasks.values().into_iter();
        let tasks_to_search: Vec<&Box<Task>> = match opt_status {
            None => tasks_iter.collect(),
            Some(status) => tasks_iter.filter(|x| x.status == status).collect(),
        };

        let mut results = Vec::new();
        for task in tasks_to_search {
            let task_repr = task.get_repr();
            if task_repr.contains(query) {
                results.push(task_repr.to_string());
            }
        }
        Ok(results)
    }

    /// Get a task's repr.
    pub fn view_task(&self, task_id: &TID) -> Result<String, String> {
        self.check_task_exists(task_id)?;
        let task = &*self.tasks.get(task_id).unwrap();
        let mut info = format!("{}\n", underline_text("Task Info"));

        info.push_str(&format!(
            "{}: {}\n",
            bold_text("name"),
            task.get_name(),
        ));
        info.push_str(&format!(
            "{}: {}\n",
            bold_text("id"),
            bold_tid(*task_id),
        ));
        info.push_str(&format!(
            "{}: {}\n",
            bold_text("status"),
            bold_text(task.get_status().to_name())
        ));
        info.push_str(&format!(
            "{}: {}",
            bold_text("created"),
            task.get_created_timestamp(),
        ));
        match task.get_desc() {
            Some(desc) => info.push_str(&format!(
                "\n{}: {}",
                bold_text("description"),
                desc,
            )),
            _ => (),
        }
        Ok(info)
    }

    /// Set a task's status.
    pub fn set_status(&mut self, task_id: &TID, status_flag: String) -> Result<(), String> {
        self.check_task_exists(task_id)?;
        let status = TaskStatus::from_status_flag(&status_flag)?;
        if status != TaskStatus::Open && self.count_available_children(task_id) > 0 {
            return Err(format!(
                "Cannot set task {} as {}; the task has open dependencies",
                bold_tid(*task_id),
                bold_text(status.to_name()),
            ));
        }
        (**self.tasks.get_mut(task_id).unwrap()).set_status(status);
        Ok(())
    }

    pub fn get_status(&mut self, task_id: &TID) -> Result<TaskStatus, String> {
        self.check_task_exists(task_id)?;
        Ok(self.tasks.get(task_id).unwrap().status)
        
    }

    /// Add the task with depends_on_id as a dependency for the task with task_id. Note that since
    /// we require the dependency graph to be acyclic, we throw an error if adding the dependency
    /// creates a cycle.
    pub fn add_dependency(&mut self, task_id: &TID, depends_on_id: &TID) -> Result<(), String> {
        self.check_task_exists(task_id)?;
        self.check_task_exists(depends_on_id)?;
        if task_id == depends_on_id {
            return Err(format!(
                "Cannot create dependency for task {} on itself.",
                bold_tid(*task_id)
            ));
        }
        match self.children.get(task_id).unwrap().contains(depends_on_id) {
            true => return Err(format!(
                        "Task {} already depends on task {}.",
                        bold_tid(*task_id),
                        bold_tid(*depends_on_id)
                    )),
            false => ()
        }
        
        if self.path_between(depends_on_id, task_id) {
            return Err(format!(
                "Adding dependency for task {} on task {} creates a cycle.",
                bold_tid(*task_id),
                bold_tid(*depends_on_id),
            ));
        }
        let this_children = self.children.get_mut(task_id).unwrap();
        this_children.push(*depends_on_id);
        let depends_on_parents = self.parents.get_mut(depends_on_id).unwrap();
        depends_on_parents.push(*task_id);
        Ok(())
    }

    /// Adds a dependency between task_id and depends_on_id. Removes depends_on_id from task_id's
    /// dependencies, adds new_id to task_id's dependencies, adds depends_on_id to new_id's
    /// dependencies. Requires that task_id has depends_on_id as a dependency.
    pub fn add_dependency_btwn(
        &mut self,
        task_id: &TID,
        new_id: &TID,
        depends_on_id: &TID
    ) -> Result<(), String> {
        self.remove_dependency(task_id, depends_on_id)?;
        self.add_dependency(task_id, new_id)?;
        self.add_dependency(new_id, depends_on_id)?;
        Ok(())
    }

    /// Remove depends_on_id as a dependency of task_id. Returns error if task_id does not depend
    /// on depends_on_id.
    pub fn remove_dependency(&mut self, task_id: &TID, depends_on_id: &TID) -> Result<(), String> {
        self.check_task_exists(task_id)?;
        self.check_task_exists(depends_on_id)?;
        let this_children = self.children.get_mut(task_id).unwrap();
        if !this_children.contains(depends_on_id) {
            return Err(format!(
                "Task {} does not depend on {}",
                bold_tid(*task_id),
                bold_tid(*depends_on_id)
            ));
        }
        this_children.retain(|child_id| child_id != depends_on_id);
        let depends_on_parents = self.parents.get_mut(depends_on_id).unwrap();
        depends_on_parents.retain(|parent_id| parent_id != task_id);
        Ok(())
    }

    /// View a task's dependencies. If no status flag is given, displays all available tasks. If a
    /// status is given, displays all dependencies with that status. If "all" is given as a status
    /// flag, displays all of the task's dependencies.
    pub fn view_dependencies(
        &self,
        task_id: &TID,
        opt_status_flag: Option<String>,
    ) -> Result<String, String> {
        let dep_ids = self.get_dependencies(task_id, opt_status_flag)?;

        let mut res = String::from(format!("dependencies for task {}:", task_id));
        for dep_id in dep_ids {
            let dep = self.tasks.get(dep_id).unwrap();
            res.push_str(&*dep.get_repr());
            res.push_str("\n");
        }
        Ok(res)
    }

    /// Get a task's dependencies. If no status flag is given, displays all available tasks. If a
    /// status is given, displays all dependencies with that status. If "all" is given as a status
    /// flag, displays all of the task's dependencies.
    pub fn get_dependencies(
        &self, 
        task_id: &TID,
        opt_status_flag: Option<String>,
    ) -> Result<Vec<&TID>, String> {
        self.check_task_exists(task_id)?;
        let (only_leaves, only_available, status_filter) = match opt_status_flag {
            None => (true, true, None),
            Some(status_flag) => match &status_flag[..] {
                "all" => (false, false, None),
                _ => {
                    let status = Some(TaskStatus::from_status_flag(&status_flag)?);
                    (false, false, status)
                }
            }
        };

        let mut visited = HashSet::new();
        Ok(self.get_dependencies_helper(
            task_id,
            only_leaves,
            only_available,
            status_filter,
            &mut visited
        ).into_iter().collect())
    }

    fn get_dependencies_helper(
        &self, 
        task_id: &TID,
        only_leaves: bool,
        only_available: bool,
        status_filter: Option<TaskStatus>,
        visited: &mut HashSet<TID>
    ) -> HashSet<&TID> {
        // Check if this task has already been visited.
        if visited.contains(task_id) {
            return HashSet::new();
        } else {
            visited.insert(task_id.clone());
        }

        // Get this task's dependencies.
        let this_children = self.children.get(task_id)
            .expect(&format!("Task with ID {} does not exist.", task_id));
        if this_children.len() == 0 {
            return HashSet::new();
        }

        let mut to_return = HashSet::new();
        for child_id in this_children {
            let num_children = self.children.get(child_id).unwrap().len();
            let mut leaf = num_children == 0;
            // If only available, define a leaf as having no available children
            if only_available {
                let num_available_children = self.count_available_children(child_id);
                leaf = leaf || num_available_children == 0;
                
            }
            let closed = self.tasks.get(child_id).unwrap().status == TaskStatus::Closed;

            // add this child to the results if:
            //  1) either the child is a leaf, or we want all tasks, and
            //  2) the child is available (not complete), or we don't want only available tasks.
            if (leaf || !only_leaves) && (!closed || !only_available) {
                match status_filter {
                    None => {
                        to_return.insert(child_id);
                    },
                    Some(status) => {
                        if status == self.tasks.get(child_id).unwrap().status {
                            to_return.insert(child_id);
                        }
                    },
                }
            }

            // if not a leaf, recurse on the child.
            if !leaf {
                to_return.extend(&self.get_dependencies_helper(
                    child_id,
                    only_leaves,
                    only_available,
                    status_filter,
                    visited
                ));
            }

        }
        to_return
    }

    /// Ensure that adding a dependency does not create a cycle.
    fn path_between(&self, u: &TID, v: &TID) -> bool {
        if u == v {
            return true;
        }
        for child_id in self.children.get(u).unwrap().iter() {
            if self.path_between(child_id, v) {
                return true;
            }
        }
        false
    }
    
    /// Count a task's number of non-completed children.
    fn count_available_children(&self, task_id: &TID) -> usize {
        let children = self.children.get(task_id).unwrap();
        let mut num_available = 0;
        for id in children {
            if self.tasks.get(&id).unwrap().status != TaskStatus::Closed {
                num_available += 1;
            }
        }
        num_available
    }

    /// Check if the task with the given TID exists.
    fn check_task_exists(&self, task_id: &TID) -> Result<(), String> {
        match self.tasks.contains_key(task_id) {
            true => Ok(()),
            false => Err(format!(
                "Task {} does not exist in the active project.",
                bold_tid(*task_id)
            ))
        }
    }

    fn get_available_tasks(&self) -> Vec<&Task> {
        let mut result = Vec::new();
        
        for task_id in self.tasks.keys() {
            let task = self.tasks.get(&task_id).unwrap();
            let num_available_children = self.count_available_children(task_id);
            let leaf = num_available_children == 0;
            let not_closed = task.status != TaskStatus::Closed;
            if leaf && not_closed {
                result.push(&**task);
            }
        }
        result
    }

}


#[derive(PartialEq, Serialize, Deserialize, Debug, Copy, Clone)]
pub enum TaskStatus {
    Open,
    InProgress,
    Closed,
}

impl TaskStatus {

    fn from_status_flag(status_flag: &str) -> Result<Self, String> {
        match status_flag {
            "open" => Ok(Self::Open),
            "in-progress" => Ok(Self::InProgress),
            "closed" => Ok(Self::Closed),
            _ => Err(format!("No such status {}", bold_text(status_flag)))
        }
    }

    fn from_opt_status_flag(opt_status_flag: Option<String>) -> Result<Option<Self>, String> {
        match opt_status_flag {
            Some(status_flag) => {
                match Self::from_status_flag(&status_flag) {
                    Ok(status) => Ok(Some(status)),
                    Err(msg) => Err(msg),
                }
            },
            None => Ok(None)
        }
    }

    pub fn to_name(&self) -> &str {
        match self {
            Self::Open => "open",
            Self::InProgress => "in-progress",
            Self::Closed => "closed",
        }
    }

}

impl ToString for TaskStatus {
        
    fn to_string(&self) -> String {
        match self {
            TaskStatus::Open => String::from("[O]"),
            TaskStatus::InProgress => String::from("[I]"),
            TaskStatus::Closed => String::from("[C]"),
        }
    }
 
}



#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct Task {
    created_timestamp: String,
    name: String,
    desc: Option<String>,
    id: TID,
    status: TaskStatus,
    repr: String,
}

impl Task {

    pub fn new(id: TID, name: String, desc: Option<String>) -> Self {
        let status = TaskStatus::Open;
        let cur_time: DateTime<Utc> = Utc::now();

        let mut new_task = Task {
            created_timestamp: cur_time.format(DATE_FORMAT).to_string(),
            id,
            repr: String::new(),
            desc,
            name,
            status,
        };
        new_task.update_repr();
        new_task
    }

    pub fn get_id(&self) -> &TID {
        &self.id
    }

    pub fn get_name(&self) -> &str {
        &self.name
    }

    pub fn get_desc(&self) -> Option<&str> {
        match &self.desc {
            Some(description) => Some(&description),
            None => None,
        }
    }

    pub fn get_created_timestamp(&self) -> &str {
        &self.created_timestamp
    }

    pub fn get_status(&self) -> &TaskStatus {
        &self.status
    }

    pub fn get_repr(&self) -> &String {
        &self.repr
    }

    pub fn set_status(&mut self, new_status: TaskStatus) {
        self.status = new_status;
        self.update_repr();
    }

    /// We cache the repr for searching.
    fn update_repr(&mut self) {
        let status_str = format!("{} {: >5}", &self.status.to_string(), &self.id);
        let mut new_repr = String::from(status_str);
        new_repr.push_str(&format!(": {}", &self.name));
        self.repr = new_repr
    }

}



#[cfg(test)]
pub mod tests {
    use super::*;

    fn get_task_reprs(tree: &TaskTree) -> Vec<&String> {
        let mut reprs = Vec::new();
        for task in tree.tasks.values().into_iter() {
            reprs.push(task.get_repr());
        }
        reprs
    }

    fn get_children_for<'a>(tree: &'a TaskTree, task_id: &'a TID) -> &'a Vec<TID> {
        tree.children.get(task_id).unwrap()
    }

    fn get_parents_for<'a>(tree: &'a TaskTree, task_id: &'a TID) -> &'a Vec<TID> {
        tree.parents.get(task_id).unwrap()
    }

    fn has_dependency(tree: &TaskTree, task_id: &TID, depends_on_id: &TID) -> bool {
        let task_dependencies = tree.children.get(task_id).expect(
                &format!("Task with ID {} not found.", task_id)
        );
        for &id in task_dependencies {
            if id == *depends_on_id {
                return true;
            }
        }
        false
    }

    fn has_parent(tree: &TaskTree, task_id: &TID, parent_id: &TID) -> bool {
        let task_parents = tree.parents.get(task_id).expect(
                &format!("Task with ID {} not found.", task_id)
        );

        for &id in task_parents {
            if id == *parent_id {
                return true;
            }
        }
        false
    }

    pub fn setup_tree() -> TaskTree {
        let mut tree = TaskTree::new();
        let tid1 = tree.add_task("Task 1".to_string(), None);
        let tid2 = tree.add_task("Task 2".to_string(), None);
        let tid3 = tree.add_task("Task 3".to_string(), None);
        let tid4 = tree.add_task("Task 4".to_string(), None);
        let tid5 = tree.add_task("Task 5".to_string(), None);
        let tid6 = tree.add_task("Task 6".to_string(), None);
        let tid7 = tree.add_task("Task 7".to_string(), None);

        // (1)-------
        //  |       |
        //  |       |
        //  |       |
        // (2)      |
        //  | \     |
        //  |  \    |
        // (3) (4) (7)
        //  |  /|   |
        //  | / |   |
        // (5) (6)---
        
        tree.add_dependency(&tid1, &tid2).unwrap();
        tree.add_dependency(&tid2, &tid3).unwrap();
        tree.add_dependency(&tid2, &tid4).unwrap();
        tree.add_dependency(&tid3, &tid5).unwrap();
        tree.add_dependency(&tid4, &tid5).unwrap();
        tree.add_dependency(&tid4, &tid6).unwrap();
        tree.add_dependency(&tid1, &tid7).unwrap();
        tree.add_dependency(&tid7, &tid6).unwrap();

        tree
    }
    
    #[test]
    fn test_add_task_and_get_desc() {
        let mut tree = setup_tree();

        let name = "Task 8";
        let task_id = tree.add_task(name.to_string(), None);

        let task_name = &tree.tasks.get(&task_id).unwrap().name;
        assert!(task_name == name);
    }

    #[test]
    fn test_remove_task() {
        let mut tree = setup_tree();

        let tid7: TID = 7;
        assert!(tree.tasks.contains_key(&tid7));

        let tid1: TID = 1;
        let tid6: TID = 6;

        let tid1_children = get_children_for(&tree, &tid1);
        let tid6_parents = get_parents_for(&tree, &tid6);

        assert!(tid1_children.contains(&tid7));
        assert!(tid6_parents.contains(&tid7));

        tree.remove_task(&tid7).unwrap();

        let tid1_children = get_children_for(&tree, &tid1);
        let tid6_parents = get_parents_for(&tree, &tid6);

        assert!(!tid1_children.contains(&tid7));
        assert!(!tid6_parents.contains(&tid7));

        assert!(!tree.tasks.contains_key(&tid7));
    }

    #[test]
    fn test_get_task_description() {
        let tree = setup_tree();
        let tid6: TID = 6;
        assert!(tree.tasks.get(&tid6).unwrap().name == "Task 6");
    }

    #[test]
    fn test_get_tasks() {
        let mut tree = setup_tree();
        let tid6: TID = 6;
        tree.set_status(&tid6, "closed".to_string()).unwrap();

        let expect_tasks = vec![
            "[O]     1: Task 1",
            "[O]     2: Task 2",
            "[O]     3: Task 3",
            "[O]     4: Task 4",
            "[O]     5: Task 5",
            "[C]     6: Task 6",
            "[O]     7: Task 7",
        ];

        let tasks = get_task_reprs(&tree);

        assert!(expect_tasks.len() == tasks.len());
        for task in tasks {
            assert!(expect_tasks.contains(&&task[..]));
        }
    }

    #[test]
    fn test_search_tasks() {
        let mut tree = setup_tree();
        let tid6: TID = 6;
        tree.set_status(&tid6, "closed".to_string()).unwrap();

        let expect_matches = vec![
            "[C]     6: Task 6",
        ];
        let matches = tree.search_tasks("[C]", None).unwrap();
        for _match in &matches {
            assert!(expect_matches.contains(&&_match[..]));
        }
        assert!(expect_matches.len() == matches.len());

        let expect_matches = vec![
            "[O]     1: Task 1",
            "[O]     2: Task 2",
            "[O]     3: Task 3",
            "[O]     4: Task 4",
            "[O]     5: Task 5",
            "[C]     6: Task 6",
            "[O]     7: Task 7",
        ];
        let matches = tree.search_tasks("Task", None).unwrap();
        for _match in &matches {
            assert!(expect_matches.contains(&&_match[..]));
        }
        assert!(expect_matches.len() == matches.len());
    }

    #[test]
    fn test_add_dependency_success() {
        let mut tree = setup_tree();
        let tid8 = tree.add_task(String::from("Task 8"), None);
        let tid1: TID = 1;
        tree.add_dependency(&tid1, &tid8).unwrap();

        let tid1_children = get_children_for(&tree, &tid1);
        let tid8_parents = get_parents_for(&tree, &tid8);

        assert!(tid1_children.contains(&tid8));
        assert!(tid8_parents.contains(&tid1));
    }

    #[test]
    #[should_panic]
    fn test_add_dependency_panic() {
        let mut tree = setup_tree();
        let tid6: TID = 6;
        let tid1: TID = 1;
        tree.add_dependency(&tid6, &tid1).unwrap();
    }

    #[test]
    fn test_add_dependency_btwn_success() {
        let mut tree = setup_tree();
        let tid8 = tree.add_task(String::from("Task 8"), None);
        let tid1: TID = 1;
        let tid2: TID = 2;
        tree.add_dependency_btwn(&tid1, &tid8, &tid2).unwrap();

        let tid1_children = get_children_for(&tree, &tid1);
        let tid2_parents = get_children_for(&tree, &tid1);
        let tid8_children = get_children_for(&tree, &tid8);
        let tid8_parents = get_parents_for(&tree, &tid8);

        assert!(!tid1_children.contains(&tid2));
        assert!(tid1_children.contains(&tid8));

        assert!(!tid2_parents.contains(&tid1));
        assert!(tid2_parents.contains(&tid8));

        assert!(tid8_children.contains(&tid2));
        assert!(tid8_parents.contains(&tid1));
    }

    #[test]
    #[should_panic]
    fn test_add_dependency_btwn_panic() {
        let mut tree = setup_tree();
        let tid1: TID = 1;
        let tid6: TID = 6;
        let tid8 = tree.add_task(String::from("Task 8"), None);
        tree.add_dependency_btwn(&tid1, &tid6, &tid8).unwrap();
    }

    #[test]
    fn test_remove_dependency() {
        let mut tree = setup_tree();
        let tid1: TID = 1;
        let tid7: TID = 7;

        let tid1_children = get_children_for(&tree, &tid1);
        assert!(tid1_children.contains(&tid7));

        let tid7_parents = get_parents_for(&tree, &tid7);
        assert!(tid7_parents.contains(&tid1));

        tree.remove_dependency(&tid1, &tid7).unwrap();

        let tid1_children = get_children_for(&tree, &tid1);
        assert!(!tid1_children.contains(&tid7));

        let tid7_parents = get_parents_for(&tree, &tid7);
        assert!(!tid7_parents.contains(&tid1));
    }

    #[test]
    fn test_get_children() {
        let tree = setup_tree();
        let tid1: TID = 1;
        let tid2: TID = 2;
        let tid7: TID = 7;

        let expect_matches = vec![tid2, tid7];
        let matches = get_children_for(&tree, &tid1);
        for _match in matches.iter() {
            assert!(expect_matches.contains(_match));
        }
        assert!(expect_matches.len() == matches.len());
    }

    #[test]
    fn test_get_parents() {
        let tree = setup_tree();
        let tid2: TID = 2;
        let tid3: TID = 3;
        let tid4: TID = 4;
        let tid5: TID = 5;

        let expect_matches = vec![tid3, tid4];
        let matches = get_parents_for(&tree, &tid5);
        for _match in matches.iter() {
            assert!(expect_matches.contains(_match));
        }
        assert!(expect_matches.len() == matches.len());

        let expect_matches = vec![tid2];
        let matches = get_parents_for(&tree, &tid4);
        for _match in matches.iter() {
            assert!(expect_matches.contains(_match));
        }
        assert!(expect_matches.len() == matches.len());
    }

    #[test]
    fn test_has_dependency() {
        let tree = setup_tree();
        let tid1: TID = 1;
        let tid2: TID = 2;
        let tid3: TID = 3;
        assert!(has_dependency(&tree, &tid1, &tid2));
        assert!(!has_dependency(&tree, &tid1, &tid3));
    }

    #[test]
    fn test_has_parent() {
        let tree = setup_tree();
        let tid1: TID = 1;
        let tid2: TID = 2;
        let tid3: TID = 3;
        assert!(has_parent(&tree, &tid2, &tid1));
        assert!(!has_parent(&tree, &tid3, &tid1));
    }

    #[test]
    fn test_get_dependencies() {
        let mut tree = setup_tree();

        let tid1: TID = 1;
        let tid2: TID = 2;
        let tid3: TID = 3;
        let tid4: TID = 4;
        let tid5: TID = 5;
        let tid6: TID = 6;
        let tid7: TID = 7;

        tree.remove_dependency(&tid1, &tid7).unwrap();
        tree.remove_dependency(&tid7, &tid6).unwrap();
        tree.add_dependency(&tid4, &tid7).unwrap();
        tree.set_status(&tid6, "closed".to_string()).unwrap();

        // (1)
        //  |
        // (2)
        //  | \
        //  |  \
        // (3) (4)
        //  |  /|\
        //  | / | \
        // (5) [6] (7)

        // here, deps should have all available dependencies
        let deps = tree.get_dependencies_helper(&tid1, false, true, None, &mut HashSet::new());

        assert!(deps.contains(&tid2));
        assert!(deps.contains(&tid3));
        assert!(deps.contains(&tid4));
        assert!(deps.contains(&tid5));
        assert!(deps.contains(&tid7));
        assert!(deps.len() == 5);

        // here, deps should have all dependencies
        let deps = tree.get_dependencies_helper(&tid1, false, false, None, &mut HashSet::new());

        assert!(deps.contains(&tid2));
        assert!(deps.contains(&tid3));
        assert!(deps.contains(&tid4));
        assert!(deps.contains(&tid5));
        assert!(deps.contains(&tid6));
        assert!(deps.contains(&tid7));
        assert!(deps.len() == 6);

        // here, deps should have only available, leaf dependencies
        let deps = tree.get_dependencies_helper(&tid1, true, true, None, &mut HashSet::new());

        assert!(deps.contains(&&tid5));
        assert!(deps.contains(&&tid7));
        assert!(deps.len() == 2);

        // here, deps should have all leaf dependencies
        let deps = tree.get_dependencies_helper(&tid1, true, false, None, &mut HashSet::new());

        assert!(deps.len() == 3);
        assert!(deps.contains(&&tid5));
        assert!(deps.contains(&&tid6));
        assert!(deps.contains(&&tid7));
    }
}
