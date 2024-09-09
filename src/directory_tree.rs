use devicons::{icon_for_file, File, Theme};
use std::fs;
use std::path::Path;
use termcolor::{Color, ColorChoice, ColorSpec, StandardStream, WriteColor};

const DIRECTORY_ICON: char = '';
const TREE_BRANCH: &str = "├── ";
const TREE_LAST_BRANCH: &str = "└── ";

pub fn handle_directory(path: &Path) -> std::io::Result<()> {
    print_root_directory(path)?;
    print_directory_contents(path)?;
    Ok(())
}

fn print_root_directory(dir: &Path) -> std::io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)).set_bold(true))?;
    println!("{}  {}", DIRECTORY_ICON, dir.display());
    stdout.reset()?;
    Ok(())
}

fn print_directory_contents(dir: &Path) -> std::io::Result<()> {
    let mut stdout = StandardStream::stdout(ColorChoice::Always);
    if dir.is_dir() {
        let mut entries: Vec<_> = fs::read_dir(dir)?.collect::<Result<_, _>>()?;
        entries.sort_by_key(|entry| {
            let path = entry.path();
            (
                !path.is_dir(),
                path.file_name()
                    .unwrap_or_default()
                    .to_string_lossy()
                    .to_string(),
            )
        });
        for (index, entry) in entries.iter().enumerate() {
            let path = entry.path();
            let is_last = index == entries.len() - 1;

            if is_last {
                print!("{}", TREE_LAST_BRANCH);
            } else {
                print!("{}", TREE_BRANCH);
            }

            if path.is_dir() {
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::Blue)).set_bold(true))?;
                println!(
                    "{}  {}",
                    DIRECTORY_ICON,
                    path.file_name().unwrap().to_string_lossy()
                );
                stdout.reset()?;
            } else {
                let file = File::new(&path);
                let icon = icon_for_file(&file, Some(Theme::Dark));
                stdout.set_color(ColorSpec::new().set_fg(Some(Color::White)))?;
                println!(
                    "{}  {}",
                    icon.icon,
                    path.file_name().unwrap().to_string_lossy()
                );
                stdout.reset()?;
            }
        }
    }
    Ok(())
}
