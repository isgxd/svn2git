use clap::Parser;

use svn2git::{
    Cli, Commands, DefaultUserInteractor, DiskStorage, HistoryCommands, HistoryManager, Result,
    SyncTool, select_or_create_config_with_interactor,
};

fn main() -> Result<()> {
    let cli = Cli::parse();

    let storage = DiskStorage::new("config.json".into());
    let mut history = HistoryManager::new(storage)?;

    match cli.command {
        Commands::Sync { svn_dir, git_dir } => {
            let interactor = DefaultUserInteractor;
            let config = select_or_create_config_with_interactor(
                svn_dir,
                git_dir,
                &mut history,
                &interactor,
            )?;
            let interactor = Box::new(DefaultUserInteractor);
            let git_operations = Box::new(config.create_git_operations());
            let tool = SyncTool::new(config, history, interactor, git_operations);
            tool.run()?;
        }
        Commands::History { command } => match command {
            HistoryCommands::List => history.list(),
            HistoryCommands::Delete { id } => history.remove_record(id)?,
        },
    }

    Ok(())
}
