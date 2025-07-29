use clap::Parser;

use svn2git::{
    Cli, Commands, DiskStorage, HistoryCommands, HistoryManager, Result, SyncTool,
    select_or_create_config_default,
};

fn main() -> Result<()> {
    let cli = Cli::parse();

    let storage = DiskStorage::new("config.json".into());
    let mut history = HistoryManager::new(storage)?;

    match cli.command {
        Commands::Sync { svn_dir, git_dir } => {
            let config = select_or_create_config_default(svn_dir, git_dir, &mut history)?;
            let tool = SyncTool::new(config, history);
            tool.run()?;
        }
        Commands::History { command } => match command {
            HistoryCommands::List => history.list(),
            HistoryCommands::Delete { id } => history.remove_record(id)?,
        },
    }

    Ok(())
}
