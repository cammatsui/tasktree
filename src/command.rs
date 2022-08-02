use crate::project::Project;
use std::io;
use std::io::Write;
use crate::tree::TID;
use ansi_term::Style;


pub const GENERAL_USAGE: &str = "usage: tasktree action [args...]";
const NEW_PROJECT_USAGE: &str = "usage: tasktree new-project project_name project_desc";
const SWITCH_PROJECT_USAGE: &str = "usage: tasktree switch-project project_name";
const REMOVE_PROJECT_USAGE: &str = "usage: tasktree rm-project project_name";
const NO_ACTIVE_MSG: &str = "No project is currently active. Run \"tasktree switch project_name\" \
                             to switch to a project.";
const NEW_TASK_USAGE: &str = "usage: tasktree new task_name [task_desc]";
const REMOVE_TASK_USAGE: &str = "usage: tasktree rm task_id";
const FIND_TASKS_USAGE: &str = "usage: tasktree find query [status]";
const VIEW_TASK_USAGE: &str = "usage: tasktree view-task task_id";
const SET_STATUS_USAGE: &str = "usage: tasktree set task_id new_status";
const ADD_DEP_USAGE: &str = "usage: tasktree add-dep task_id [dependency_ids...]";
const ADD_DEP_BTWN_USAGE: &str = "usage: tasktree add-dep-btwn task_id btwn_id dependency_id";
const REMOVE_DEP_USAGE: &str = "usage: tasktree rm-dep task_id dependency_id";
const VIEW_DEPS_USAGE: &str = "usage: tasktree view-deps task_id [status]";


/// Enum representing an action the user would like to execute.
#[derive(PartialEq, Debug)]
enum Action {
    NewProject,
    RemoveProject,
    ListProjects,
    ViewProject,
    SwitchProject,
    NewTask,
    RemoveTask,
    ViewTasks,
    FindTasks,
    ViewTask,
    SetStatus,
    AddDep,
    AddDepBtwn,
    RemoveDep,
    ViewDeps,
}

impl Action {

    fn from_cmdline_arg(arg: &str) -> Result<Self, String> {
        match arg {
            "new-project" => Ok(Self::NewProject),
            "rm-project" => Ok(Self::RemoveProject),
            "list-projects" => Ok(Self::ListProjects),
            "view-project" => Ok(Self::ViewProject),
            "switch" => Ok(Self::SwitchProject),
            "new" => Ok(Self::NewTask),
            "rm" => Ok(Self::RemoveTask),
            "view" => Ok(Self::ViewTasks),
            "find" => Ok(Self::FindTasks),
            "view-task" => Ok(Self::ViewTask),
            "set" => Ok(Self::SetStatus),
            "add-dep" => Ok(Self::AddDep),
            "add-dep-btwn" => Ok(Self::AddDepBtwn),
            "rm-dep" => Ok(Self::RemoveDep),
            "view-deps" => Ok(Self::ViewDeps),
            _ => Err(format!("no action \"{}\"", arg)),
        }
    }

}


/// Enum representing a user's command.
pub struct Command {
    action: Action,
    args: Vec<String>,
}

impl Command {

    pub fn from_args(args: Vec<String>) -> Result<Self, String> {
        if args.len() < 1 {
            return Err(GENERAL_USAGE.to_string());
        }

        let action = Action::from_cmdline_arg(&args[0])?;
        let command_args = match args.len() {
            1 => Vec::new(),
            _ => args[1..].to_vec(),
        };

        Ok(Command{ action, args: command_args })
    }

    /// Run this command.
    pub fn execute(&self) -> Result<String, String> {
        match self.action {
            Action::NewProject => self.new_project_action(),
            Action::RemoveProject => self.remove_project_action(),
            Action::ListProjects => self.list_projects_action(),
            Action::ViewProject => self.view_project_action(),
            Action::SwitchProject => self.switch_project_action(),
            Action::NewTask => self.new_task_action(),
            Action::RemoveTask => self.remove_task_action(),
            Action::ViewTasks => self.view_tasks_action(),
            Action::FindTasks => self.find_tasks_action(),
            Action::ViewTask => self.view_task_action(),
            Action::SetStatus => self.set_status_action(),
            Action::AddDep => self.add_dep_action(),
            Action::AddDepBtwn => self.add_dep_btwn_action(),
            Action::RemoveDep => self.remove_dep_action(),
            Action::ViewDeps => self.view_deps_action(),
        }
    }

    /// Create a new project with the given project name and description. If the project already
    /// exists, prompt the user ("y"/"n") to confirm replacement.
    fn new_project_action(&self) -> Result<String, String> {
        self.check_args_len(2, NEW_PROJECT_USAGE)?;

        let project_name = &self.args[0];
        let project_desc = &self.args[1];
        let already_exists_msg = &format!("The project {} already exists. Are you sure you would \
                                         like to delete this project and replace it with a new \
                                         one (y/n)? ", project_name);

        let mut replace_project = true;
        if Project::exists(project_name)? {
            let user_input = Self::get_user_input(already_exists_msg, vec!["y", "n"]);
            replace_project = match &user_input[..] {
                "y" => true,
                "n" => false,
                _ => panic!("Invalid user input allowed."),
            };
        }

        if replace_project {
            Project::new(project_name.to_string(), project_desc.to_string());
            return Ok(format!("Successfully created project {}.", project_name));
        }

        Ok(format!("Did not create project {}.", project_name))
    }

    /// Remove the project with the given name. If the project does not exist, return an error
    /// informing the user of this. Otherwise, prompt the user to confirm ("y"/"n") to confirm the
    /// removal, and remove it if "y".
    fn remove_project_action(&self) -> Result<String, String> {
        self.check_args_len(1, REMOVE_PROJECT_USAGE)?;

        let project_name = &self.args[0];
        if !Project::exists(project_name)? {
            return Err(format!("There is no project named {}.", project_name))
        }

        let prompt_msg = &format!("Are you sure you want to delete the project {}? This operation \
                                   cannot be undone (y/n) ", project_name);
        match &Self::get_user_input(prompt_msg, vec!["y", "n"])[..] {
            "y" =>  {
                Project::remove(project_name)?;
                return Ok(format!("Successfully removed project {}.", project_name));
            },
            "n" => Ok(format!("Did not remove project {}.", project_name)),
            _ => panic!("Disallowed input provided"),
        }
    }

    /// List existing tasktree project names.
    fn list_projects_action(&self) -> Result<String, String> {
        let proj_list = Project::get_project_names()?;
        if proj_list.len() == 0 {
            return Err("no tasktree projects. create one: \"tasktree new-project\"".to_string());
        }
        let mut result = String::from(format!(
            "{}",
            bold_text(&underline_text("tasktree projects:"))
        ));

        for proj_name in proj_list {
            result.push_str(&format!("\n{}", &proj_name));
        }
        Ok(result)
    }

    // Provider a summary of the active project. If there is no active project, return an error
    // message informing the user of this. Otherwise, return a summary of the active project.
    fn view_project_action(&self) -> Result<String, String> {
        match Project::get_active() {
            None => Err(NO_ACTIVE_MSG.to_string()),
            Some(proj_name) => {
                let proj = Project::load(&proj_name)?;
                let mut info = format!("{}\n", underline_text("Project Info"));

                info.push_str(&format!(
                    "{}: {}\n",
                    bold_text("name"),
                    proj_name,
                ));
                info.push_str(&format!(
                    "{}: {}\n",
                    bold_text("created"),
                    proj.get_created_timestamp(),
                ));
                info.push_str(&format!(
                    "{}: {}\n",
                    bold_text("modified"),
                    proj.get_modified_timestamp(),
                ));
                info.push_str(&format!(
                    "{}: {}",
                    bold_text("description"),
                    proj.get_desc(),
                ));
                Ok(info)
            }
        }
    }

    /// Switch the active project to the project with the given name. If no such project exists,
    /// return an error messaging informing the user. If successful, return a message confirming
    /// that the active project has been switched.
    fn switch_project_action(&self) -> Result<String, String> {
        self.check_args_len(1, SWITCH_PROJECT_USAGE)?;
        let project_name = &self.args[0];
        Project::set_active(project_name)?;

        Ok(format!("Set {} as active project.", bold_text(project_name)))
    }

    /// Create a task in the active project with the given name and optional description. If
    /// anything fails, returns appropriate error message. Otherwise, create the task, save the
    /// project, and return a message confirming that the new task was created.
    fn new_task_action(&self) -> Result<String, String> {
        self.check_args_len(1, NEW_TASK_USAGE)?;
        let task_name = &self.args[0];
        let task_desc = self.parse_optional_argument(1);
        
        let mut proj = Self::load_active_project()?;
        let tasks = proj.get_tree_mut();
        let task_id = tasks.add_task(task_name.to_string(), task_desc);
        proj.save()?;

        Ok(format!("Created task {} with id {}.", task_name, task_id))
    }

    /// Remove the task with the given id from the active project. If such a task does not exist,
    /// return an error message indicating this to the user. Otherwise, require the user to confirm
    /// ("y"/"n") to remove the task. If "y", deletes the task and informs the user.
    fn remove_task_action(&self) -> Result<String, String> {
        self.check_args_len(1, REMOVE_TASK_USAGE)?;
        let task_id = Self::parse_as_task_id(&self.args[0])?;
        let mut proj = Self::load_active_project()?;
        let tasks = proj.get_tree_mut();
        let task_repr = match tasks.get_task_repr(&task_id) {
            Some(task_repr) => task_repr,
            None => return Err(format!(
                "There is no task for the active project with id {}.",
                task_id
            )),
        };

        let prompt = format!(
            "Are you sure you want to remove the task '{}' from the active project (y/n)? ",
            task_repr
        );
        let user_input = Self::get_user_input(&prompt, vec!["y", "n"]);
        match &user_input[..] {
            "y" => {
                tasks.remove_task(&task_id)?;
                proj.save()?;
                Ok(format!("Successfully removed task {}.", bold_tid(task_id)))
            },
            "n" => Ok(format!("Did not remove task {}.", bold_tid(task_id))),
            _ => panic!("Invalid user input"),
        }
    }

    /// View the tasks in the active project which match the given status flag. By default, the
    /// status flag is "available". If there are no matching tasks, inform the user.
    fn view_tasks_action(&self) -> Result<String, String> {
        let proj = Self::load_active_project()?;
        let tasks = proj.get_tree();
        let mut result = String::new();
        let status_flag = self.parse_optional_argument(0);
        let status_flag_name = match &status_flag {
            None => "available",
            Some(x) => &x,
        };

        let matches = tasks.view_tasks(status_flag.clone())?;
        if matches.len() == 0 {
            return Err(format!(
                "no {} tasks in project {}",
                bold_text(&status_flag_name),
                bold_text(proj.get_name()),
            ));
        }
        result.push_str(&format!(
            "{} tasks in project {}:",
            bold_text(&status_flag_name),
            bold_text(proj.get_name()),
        ));
        for _match in matches {
            result.push_str("\n");
            result.push_str(&_match.get_repr());
        }

        Ok(result)
    }

    /// Find tasks in the active project which match the provided query and the optionally provided
    /// status. If no tasks match the query, inform the user.
    fn find_tasks_action(&self) -> Result<String, String> {
        self.check_args_len(1, FIND_TASKS_USAGE)?;
        let proj = Self::load_active_project()?;
        let tasks = proj.get_tree();
        let query = self.args[0].to_string();
        let status_flag = self.parse_optional_argument(1);
        let status_flag_name = match &status_flag {
            None => "all",
            Some(x) => &x,
        };
        let mut result = String::new();
        let matches = tasks.search_tasks(&query, status_flag.clone())?;
        if matches.len() == 0 {
            result.push_str(&format!(
                "no {} tasks for query '{}' in project {}",
                bold_text(&status_flag_name),
                bold_text(&query),
                bold_text(proj.get_name()),
            ));
            return Err(result);
        }

        result.push_str(&format!(
            "{} tasks for query '{}' in project {}:\n",
            bold_text(&status_flag_name),
            bold_text(&query),
            bold_text(proj.get_name()),
        ));
        for _match in matches {
            result.push_str(&_match);
            result.push_str("\n");
        }
        Ok(result.trim().to_string())
    }

    /// View a detailed summary of the task with the given ID. If no such task exists, inform the
    /// user with an error message.
    fn view_task_action(&self) -> Result<String, String> {
        self.check_args_len(1, VIEW_TASK_USAGE)?;
        let task_id = Self::parse_as_task_id(&self.args[0])?;
        let proj = Self::load_active_project()?;
        let tasks = proj.get_tree();
        tasks.view_task(&task_id)
    }

    /// Set the task with the given id's status to the given status.
    fn set_status_action(&self) -> Result<String, String> {
        self.check_args_len(2, SET_STATUS_USAGE)?;
        let task_id = Self::parse_as_task_id(&self.args[0])?;
        let status = &self.args[1];

        let mut proj = Self::load_active_project()?;
        let tasks = proj.get_tree_mut();
        tasks.set_status(&task_id, status.to_string())?;
        proj.save()?;
        Ok(format!("Set task {}'s status to {}.", 
            bold_tid(task_id),
            bold_text(status)
        ))
    }

    /// Add a dependency of the task with the first provided task id (task_id) on the tasks with 
    /// the provided other task ids (depends_on_id). Requires that this does not create a cycle.
    fn add_dep_action(&self) -> Result<String, String> {
        self.check_args_len(2, ADD_DEP_USAGE)?;
        let task_id = Self::parse_as_task_id(&self.args[0])?;

        let mut dep_ids = Vec::new();
        for dep_id_str in self.args[1..].into_iter() {
            dep_ids.push(Self::parse_as_task_id(dep_id_str)?);
        }

        let mut proj = Self::load_active_project()?;
        let tasks = proj.get_tree_mut();

        let mut result = if dep_ids.len() == 1 {
            String::from("Added task ")
        } else {
            String::from("Added tasks ")
        };

        for dep_id in &dep_ids {
            tasks.add_dependency(&task_id, &dep_id)?;
            result.push_str(&format!("{} ", bold_tid(*dep_id)));
        }

        if dep_ids.len() == 1 {
            result.push_str(&format!("as a dependency for task {}.", bold_tid(task_id)));
        } else {
            result.push_str(&format!("as dependencies for task {}.", bold_tid(task_id)));
        };

        proj.save()?;
        Ok(result)
    }

    /// Takes three task ids: task_id, new_id, and depends_on_id. Requires that task_id depends on
    /// depends_on_id. Then removes this dependency, and add a dependencies for task_id on new_id 
    /// and for new_id on depends_on_id.
    fn add_dep_btwn_action(&self) -> Result<String, String> {
        self.check_args_len(3, ADD_DEP_BTWN_USAGE)?;
        let task_id = Self::parse_as_task_id(&self.args[0])?;
        let new_id = Self::parse_as_task_id(&self.args[1])?;
        let depends_on_id = Self::parse_as_task_id(&self.args[2])?;
        let mut proj = Self::load_active_project()?;
        let tasks = proj.get_tree_mut();
        tasks.add_dependency_btwn(&task_id, &new_id, &depends_on_id)?;

        proj.save()?;
        Ok(format!("Added task {} between {} and {}.", new_id, task_id, depends_on_id))
    }

    /// Removes a of task_id on dependency_id if the dependency and both tasks exist.
    fn remove_dep_action(&self) -> Result<String, String> {
        self.check_args_len(2, REMOVE_DEP_USAGE)?;
        let task_id = Self::parse_as_task_id(&self.args[0])?;
        let dependency_id = Self::parse_as_task_id(&self.args[1])?;

        let mut proj = Self::load_active_project()?;
        let tasks = proj.get_tree_mut();
        tasks.remove_dependency(&task_id, &dependency_id)?;

        Ok(format!("Removed dependency of task {} on task {}.", task_id, dependency_id))
    }

    /// View the dependencies for the provided task id which match the given status flag. By 
    /// default, the status flag is "available". If there are no matching tasks, informs the user.
    fn view_deps_action(&self) -> Result<String, String> {
        self.check_args_len(1, VIEW_DEPS_USAGE)?;
        let task_id = Self::parse_as_task_id(&self.args[0])?;
        let opt_status_flag = if self.args.len() > 1 {
            Some(self.args[1].to_string())
        } else {
            None
        };
        let status_flag_name = match opt_status_flag {
            None => "available".to_string(),
            Some(ref x) => x.clone(),
        };

        let proj = Self::load_active_project()?;
        let tree = proj.get_tree();
        let dep_ids = tree.get_dependencies(&task_id, opt_status_flag)?;
        let mut result = String::new();
        if dep_ids.len() == 0 {
            return Ok(format!(
                "no {} dependencies task {}",
                bold_text(&status_flag_name),
                bold_tid(task_id),
            ));
        }
        result.push_str(&format!(
            "{} dependencies for task {}:",
            bold_text(&status_flag_name),
            bold_tid(task_id),
        ));
        for dep_id in dep_ids {
            result.push_str("\n");
            result.push_str(&tree.get_task_repr(dep_id).unwrap());
        }
        Ok(result)
    }

    /// Print the prompt and get user input while the user's input is not in `allowed_vals`.
    fn get_user_input(prompt: &str, allowed_vals: Vec<&str>) -> String {
        let mut input;
        loop {
            print!("{}", prompt);
            io::stdout().flush().unwrap();
            input = String::new();
            io::stdin().read_line(&mut input)
                .expect("Error getting user input");
            input = input.trim().to_string();
            if allowed_vals.contains(&&input[..]) {
                break;
            }
        }
        input
    }

    fn parse_as_task_id(arg: &str) -> Result<TID, String> {
        match arg.parse() {
            Ok(result) => Ok(result),
            _ => return Err("task_id must be a positive integer.".to_string()),
        }
        
    }

    fn load_active_project() -> Result<Project, String> {
        let active = Project::get_active();
        match active {
            None => return Err(NO_ACTIVE_MSG.to_string()),
            Some(active_name) => {
                if !Project::exists(&active_name)? {
                    return Err(NO_ACTIVE_MSG.to_string());
                } else {
                    return Ok(Project::load(&active_name)?);
                }
            }
        }
    }

    fn check_args_len(&self, args_len: usize, usage: &str) -> Result<(), String> {
        match self.args.len() >= args_len {
            false => return Err(usage.to_string()),
            _ => Ok(()),
        }
    }

    fn parse_optional_argument(&self, idx: usize) -> Option<String> {
        if &self.args.len() < &(idx+1) {
            None
        } else {
            Some(self.args[idx].clone())
        }
    }

}

pub fn bold_text(text: &str) -> String {
    format!("{}", Style::new().bold().paint(text))
}

pub fn underline_text(text: &str) -> String {
    format!("{}", Style::new().underline().paint(text))
}

pub fn bold_tid(tid: TID) -> String {
    bold_text(&tid.to_string())
}
