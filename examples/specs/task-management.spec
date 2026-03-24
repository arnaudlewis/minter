spec task-management v1.0.0
title "Task Management"

description
  Users can create, list, complete, and delete tasks within their account.

motivation
  Core feature of the application — users need to manage their personal task list.

nfr
  performance

behavior create-task [happy_path]
  "An authenticated user can create a new task"

  nfr
    performance#api-response-time
    performance#throughput

  given
    @user = User { id: "user-1", authenticated: true }

  when create-task
    title = "Buy groceries"
    user_id = @user.id

  then returns task
    assert id is_present
    assert title == "Buy groceries"
    assert completed == false

behavior list-tasks [happy_path]
  "An authenticated user can list their tasks"

  nfr
    performance#api-response-time
    performance#no-unbounded-queries

  given
    @user = User { id: "user-1", authenticated: true }
    @task = Task { id: "task-1", title: "Buy groceries", user_id: "user-1" }

  when list-tasks
    user_id = @user.id

  then returns task-list
    assert count >= 1

behavior complete-task [happy_path]
  "A user can mark a task as completed"

  nfr
    performance#api-response-time

  given
    @user = User { id: "user-1", authenticated: true }
    @task = Task { id: "task-1", completed: false, user_id: "user-1" }

  when complete-task
    task_id = @task.id
    user_id = @user.id

  then returns task
    assert completed == true

behavior create-task-unauthenticated [error_case]
  "Task creation fails without authentication"

  given
    No authenticated user context

  when create-task
    title = "Buy groceries"

  then returns error
    assert status == "unauthorized"

behavior complete-nonexistent-task [edge_case]
  "Completing a task that does not exist returns not found"

  given
    @user = User { id: "user-1", authenticated: true }

  when complete-task
    task_id = "nonexistent"
    user_id = @user.id

  then returns error
    assert status == "not_found"

depends on user-auth >= 1.0.0
