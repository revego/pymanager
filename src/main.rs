use clap::{Parser, Subcommand};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};
use tui::{
    backend::CrosstermBackend,
    layout::{Constraint},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Cell, Row, Table},
    Terminal,
};
use crossterm::event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode};
use crossterm::execute;
use crossterm::terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen};

#[derive(Parser)]
#[command(name = "pymanager")]
#[command(about = "A tool to manage Python environments and projects", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all Python versions available on the system
    ListPythonVersions,
    /// List all projects worked on by a specific Python version
    ListPythonProjects { version: String },
    /// Add a project to the log for a specific Python version
    AddProject { version: String, project: String },
    /// Show projects in a table
    ShowTable,
}

#[derive(Serialize, Deserialize, Clone)]
struct Project {
    name: String,
    created_at: u64,
    last_accessed: u64,
}

#[derive(Serialize, Deserialize)]
struct ProjectLog {
    version: String,
    projects: Vec<Project>,
}

fn get_python_versions() -> Vec<String> {
    let mut versions = Vec::new();
    let paths = vec!["/usr/bin", "/usr/local/bin"];

    for path in paths {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_string_lossy();

                    if file_name_str.starts_with("python") {
                        let re = Regex::new(r"python(\d+)\.(\d+)").unwrap();
                        if let Some(caps) = re.captures(&file_name_str) {
                            let version = format!("{}.{}", &caps[1], &caps[2]);
                            if !versions.contains(&version) {
                                //let version_clone = version.clone(); // Clonare la versione prima di spostarla nel vettore
                                versions.push(version);
                                //println!("Intercepted Python version: {}", version);
                            }
                        }
                    }
                }
            }
        }
    }

    versions
}


fn get_python_versions2() -> Vec<String> {
    let mut versions = Vec::new();
    let paths = vec!["/usr/bin", "/usr/local/bin"];

    for path in paths {
        if let Ok(entries) = fs::read_dir(path) {
            for entry in entries {
                if let Ok(entry) = entry {
                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_str().unwrap_or("");

                    if file_name_str.starts_with("python") {
                        let re = Regex::new(r"python\d+\.\d+").unwrap();
                        if let Some(caps) = re.captures(file_name_str) {
                            let version = caps.get(0).unwrap().as_str().to_string();
                            if !versions.contains(&version) {
                                versions.push(version);
                            }
                        }
                    }
                }
            }
        }
    }
    versions
}

fn load_project_log(version: &str) -> ProjectLog {
    let path = format!("/var/log/pymanager/{}.json", version);
    if Path::new(&path).exists() {
        let data = fs::read_to_string(path).unwrap();
        serde_json::from_str(&data).unwrap()
    } else {
        ProjectLog {
            version: version.to_string(),
            projects: vec![],
        }
    }
}

fn save_project_log(log: &ProjectLog) {
    let dir = "/var/log/pymanager";
    fs::create_dir_all(dir).unwrap();
    let path = format!("{}/{}.json", dir, log.version);
    let data = serde_json::to_string(log).unwrap();
    fs::write(path, data).unwrap();
}

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs()
}

fn list_python_versions() {
    let versions = get_python_versions();
    if versions.is_empty() {
        println!("No Python versions found.");
    } else {
        println!("Python versions found:");
        for version in versions {
            println!("{}", version);
        }
    }
}

fn list_python_projects(version: &str) {
    let log = load_project_log(version);
    if log.projects.is_empty() {
        println!("No projects found for Python version {}", version);
    } else {
        println!("Projects worked on by Python version {}:", version);
        for project in log.projects {
            println!(
                "{} (created at {}, last accessed at {})",
                project.name, project.created_at, project.last_accessed
            );
        }
    }
}

fn add_project(version: &str, project_name: &str) {
    let mut log = load_project_log(version);
    let timestamp = current_timestamp();

    if log.projects.iter().any(|p| p.name == project_name) {
        println!(
            "Project '{}' already exists for Python version {}",
            project_name, version
        );
    } else {
        log.projects.push(Project {
            name: project_name.to_string(),
            created_at: timestamp,
            last_accessed: timestamp,
        });
        save_project_log(&log);
        println!(
            "Project '{}' added to Python version {}",
            project_name, version
        );
    }
}

fn show_table() -> Result<(), Box<dyn std::error::Error>> {
    enable_raw_mode()?;
    let mut stdout = std::io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let versions = get_python_versions();

    for version in &versions {
        println!("Python version listed: {}", version);
    }

    let mut rows: Vec<Row> = Vec::new();

    for version in versions {
        let log = load_project_log(&version);
        for project in log.projects {
            rows.push(Row::new(vec![
                    Cell::from(version.clone()),
                    Cell::from(project.name.clone()),
                    Cell::from(format!("{}", project.created_at)),
                    Cell::from(format!("{}", project.last_accessed)),
            ]));
        }
    }

    loop {
        terminal.draw(|f| {
            let size = f.size();
            let block = Block::default().borders(Borders::ALL).title("Python Projects");
            let table = Table::new(rows.clone())
                .block(block)
                .header(Row::new(vec![
                        Cell::from("Version").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                        Cell::from("Project").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                        Cell::from("Created At").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                        Cell::from("Last Accessed").style(Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
                ]))
                .widths(&[
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                    Constraint::Percentage(25),
                ]);
            f.render_widget(table, size);
        })?;

        if let Event::Key(key) = event::read()? {
            if key.code == KeyCode::Char('q') {
                break;
            }
        }
    }

    disable_raw_mode()?;
    execute!(terminal.backend_mut(), LeaveAlternateScreen, DisableMouseCapture)?;
    terminal.show_cursor()?;

    Ok(())
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::ListPythonVersions => {
            list_python_versions();
        }
        Commands::ListPythonProjects { version } => {
            list_python_projects(version);
        }
        Commands::AddProject { version, project } => {
            add_project(version, project);
        }
        Commands::ShowTable => {
            show_table().unwrap();
        }
    }
}

