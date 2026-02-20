use hackshell::Hackshell;

#[test]
fn test_fork_inherits_environment() {
    let parent = Hackshell::new("parent> ").unwrap();

    // Set some environment variables in the parent
    parent.set_var("foo", "bar");
    parent.set_var("count", "42");
    parent.set_var("name", "hackshell");

    // Fork the shell
    let child = parent.fork("child> ").unwrap();

    // Child should have inherited all environment variables
    assert_eq!(child.get_var("foo"), Some("bar".to_string()));
    assert_eq!(child.get_var("count"), Some("42".to_string()));
    assert_eq!(child.get_var("name"), Some("hackshell".to_string()));
}

#[test]
fn test_fork_child_env_independent_from_parent() {
    let parent = Hackshell::new("parent> ").unwrap();
    parent.set_var("shared", "original");

    let child = parent.fork("child> ").unwrap();

    // Modify the child's environment
    child.set_var("shared", "modified_by_child");
    child.set_var("child_only", "exists");

    // Parent should not be affected
    assert_eq!(parent.get_var("shared"), Some("original".to_string()));
    assert_eq!(parent.get_var("child_only"), None);

    // Child should have the modified values
    assert_eq!(child.get_var("shared"), Some("modified_by_child".to_string()));
    assert_eq!(child.get_var("child_only"), Some("exists".to_string()));
}

#[test]
fn test_fork_parent_env_changes_do_not_affect_child() {
    let parent = Hackshell::new("parent> ").unwrap();
    parent.set_var("var", "before_fork");

    let child = parent.fork("child> ").unwrap();

    // Modify parent's environment after fork
    parent.set_var("var", "after_fork");
    parent.set_var("parent_only", "exists");

    // Child should have the value from fork time
    assert_eq!(child.get_var("var"), Some("before_fork".to_string()));
    assert_eq!(child.get_var("parent_only"), None);
}

#[test]
fn test_fork_child_has_builtin_commands() {
    let parent = Hackshell::new("parent> ").unwrap();
    let child = parent.fork("child> ").unwrap();

    // Child should have all built-in commands
    assert!(child.feed_line("help").is_ok());
    assert!(child.feed_line("env").is_ok());

    // Set and get should work
    child.feed_line("set test_var test_value").unwrap();
    assert_eq!(child.get_var("test_var"), Some("test_value".to_string()));
}

#[test]
fn test_fork_child_has_separate_task_pool() {
    let parent = Hackshell::new("parent> ").unwrap();
    let child = parent.fork("child> ").unwrap();

    // Spawn a task in the parent
    parent.spawn("parent_task", |_run| {
        std::thread::sleep(std::time::Duration::from_millis(100));
        None
    });

    // Child should not see the parent's task
    let child_tasks = child.get_tasks();
    assert!(child_tasks.is_empty());

    // Parent should see its own task
    let parent_tasks = parent.get_tasks();
    assert_eq!(parent_tasks.len(), 1);
    assert_eq!(parent_tasks[0].name, "parent_task");

    // Clean up
    parent.terminate("parent_task").unwrap();
}

#[test]
fn test_fork_multiple_children() {
    let parent = Hackshell::new("parent> ").unwrap();
    parent.set_var("base", "value");

    let child1 = parent.fork("child1> ").unwrap();
    let child2 = parent.fork("child2> ").unwrap();

    // Both children should have inherited the environment
    assert_eq!(child1.get_var("base"), Some("value".to_string()));
    assert_eq!(child2.get_var("base"), Some("value".to_string()));

    // Modifications to one child should not affect the other
    child1.set_var("base", "child1_value");
    child2.set_var("base", "child2_value");

    assert_eq!(child1.get_var("base"), Some("child1_value".to_string()));
    assert_eq!(child2.get_var("base"), Some("child2_value".to_string()));
    assert_eq!(parent.get_var("base"), Some("value".to_string()));
}

#[test]
fn test_fork_chain() {
    let grandparent = Hackshell::new("gp> ").unwrap();
    grandparent.set_var("level", "0");

    let parent = grandparent.fork("p> ").unwrap();
    parent.set_var("level", "1");

    let child = parent.fork("c> ").unwrap();
    child.set_var("level", "2");

    // Each shell should have its own level
    assert_eq!(grandparent.get_var("level"), Some("0".to_string()));
    assert_eq!(parent.get_var("level"), Some("1".to_string()));
    assert_eq!(child.get_var("level"), Some("2".to_string()));
}

#[test]
fn test_fork_with_empty_env() {
    let parent = Hackshell::new("parent> ").unwrap();

    // Fork without setting any env vars
    let child = parent.fork("child> ").unwrap();

    // Child's environment should also be empty
    assert!(child.env().is_empty());
}

#[test]
fn test_fork_preserves_case_insensitive_vars() {
    let parent = Hackshell::new("parent> ").unwrap();
    parent.set_var("MyVar", "value");

    let child = parent.fork("child> ").unwrap();

    // Variables are stored lowercase, so both should work
    assert_eq!(child.get_var("myvar"), Some("value".to_string()));
    assert_eq!(child.get_var("MYVAR"), Some("value".to_string()));
}
