use std::io::{self, Write};

fn main() {
    let mut todos: Vec<String> = Vec::new();

    loop {
        println!("\n--- Todo App ---");
        println!("l. List Todos");
        println!("a. Add Todo");
        println!("r. Remove Todo");
        println!("q. Exit");
        print!("Select action: ");
        std::io::stdout().flush().unwrap();

        let mut choice = String::new();

        io::stdin().read_line(&mut choice).unwrap();

        match choice.trim() {
            "l" => view_todos(&todos),
            "a" => add_todo(&mut todos),
            "r" => remove_todo(&mut todos),
            "q" => break,
            _ => print!("Invalid choice"),
        };
    }
}

fn view_todos(todos: &Vec<String>) {
    if todos.is_empty() {
        println!("No items on the list");
    } else {
        for (i, task) in todos.iter().enumerate() {
            println!("{}: {}\n", i, task);
        }
    }
}

fn add_todo(todos: &mut Vec<String>) {
    print!("Describe the task: ");
    std::io::stdout().flush().unwrap();

    let mut task = String::new();

    io::stdin()
        .read_line(&mut task)
        .expect("Couldn't read input");

    todos.push(task.trim().to_string());
}

fn remove_todo(todos: &mut Vec<String>) {
    if todos.is_empty() {
        return;
    }

    view_todos(todos);
    print!("Select a task to remove: ");
    std::io::stdout().flush().unwrap();

    let mut index = String::new();

    io::stdin().read_line(&mut index).unwrap();

    match index.trim().parse::<usize>() {
        Ok(num) if 0.ge(&num) && num < todos.len() => {
            todos.remove(num);
        }
        _ => {
            println!("Enter a valid number");
        }
    };
}
