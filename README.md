# TaskTree

## Overview

TaskTree is a CLI task and project management program. 

Features:
- Create and remove projects (project metadata + a dependency graph of tasks)
- Set active project
- Create tasks
- Add, remove, and view task dependencies
    - Add tasks between two tasks
- Mark tasks as open, closed, or, in-progress
- View tasks with a given status
- Search project tasks

## Task Statuses

Tasks can have one of the following statuses: *open*, *in-progress*, or *closed*.
Note: We call a task *available* if it is not *closed* and all of its children are *closed*.

## Commands

The general usage is:
`tasktree action [args...]`

### Project Commands

- New project: `tasktree new-project project_name project_desc`
    - Creates a new project. Checks whether the given project name exists, and if so, asks the user
      to confirm deletion of the old project.
- Remove project: `tasktree new-project project_name project_desc`
    - Deletes the given project after asking for user confirmation.
- List projects: `tasktree list-projects`
    - Lists all projects.
- View active project: `tasktree view-project project_name`
    - Gives a summary of the given project (default is active), including the project's name, 
      description, and *available* tasks.
- Switch to project: `tasktree switch project_name`
    - Switches the active project to project named "project_name".

### Task Commands

- New task: `tasktree new task_name [task_desc]`
    - Create a task in the active project with the given name and optionally provided description.
      Displays the newly created task's tid. The new task is initialized with the status *open*.
- Remove task: `tasktree rm task_id `
    - Removes the task from the active project after asking for confirmation. 
- View tasks (by status): `tasktree view [status|"all"]`
    - View the active project's tasks with an optional status filter. If a status is not provided,
      the active project's available tasks are displayed. If "all" is provided, all of the active
      project's tasks are displayed.
- Search tasks: `tasktree find query [status]`
    - Search the active project's tasks using the provided query. If an optional status parameter
      is provided, searches only tasks with the given status.
- View task: `tasktree view-task task_id`
    - View a summary of the task with the given tid, and displays the task's available 
      dependencies.
- Set task status: `tasktree set task_id new_status`
    - Set the given task's status. If the given task's parent now has no `not-completed` 
      children, informs the user that this parent is now available. 
- Add dependency: `tasktree add-dep task_id [dependency_ids...]`
    - Add a dependency for the task with task_tid on the task with dependency_tid. Errs out to the
      user if this creates a cycle.
- Add dependency between two tasks: `tasktree add-dep-btwn task_id btwn_id dependency_id`
    - Removes the dependency of task_id on dependency_id. Then adds btwn_id as a dependency for
      task_id, and adds a dependency_id as a dependency for btwn_id.
- Remove dependency: `tasktree rm-dep task_id dependency_id`
    - Remove the dependency for the task with task_tid on the task with dependency_tid
- View dependencies: `tasktree view-deps task_id [status|"all"]`
    - View the given task's dependencies. If no status flag is given, displays available tasks. If
      a status is given, displays all dependencies with that status. If "all" is given, displays
      all of the task's dependencies.
