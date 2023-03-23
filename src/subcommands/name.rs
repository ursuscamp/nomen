use crate::config::NameSubcommand;

pub async fn name(cmd: &NameSubcommand) -> anyhow::Result<()> {
    match cmd {
        NameSubcommand::New {
            name,
            children,
            privkey,
        } => new(name, children, privkey.as_ref()).await?,
    }

    Ok(())
}

async fn new(name: &str, children: &[String], privkey: Option<&String>) -> anyhow::Result<()> {
    todo!()
}
