use crate::serenity::model::interactions::message_component::ButtonStyle;
use crate::{Context, Result};

#[derive(Debug, poise::SlashChoiceParameter)]
pub enum VersionChoice {
    Legacy,
    Renewed,
}

impl VersionChoice {
    /// Curseforge project ID for the LOTR Mod Renewed
    const CURSEFORGE_ID_RENEWED: u64 = 406893;
    /// Curseforge project ID for the LOTR Mod Legacy
    const CURSEFORGE_ID_LEGACY: u64 = 423748;

    pub const fn id(&self) -> u64 {
        match self {
            VersionChoice::Legacy => Self::CURSEFORGE_ID_LEGACY,
            VersionChoice::Renewed => Self::CURSEFORGE_ID_RENEWED,
        }
    }
}

impl Default for VersionChoice {
    fn default() -> Self {
        Self::Legacy
    }
}

/// Get a link to download the mod's latest version on Curseforge
#[poise::command(slash_command)]
pub async fn download(
    ctx: Context<'_>,
    #[description = "The version to get a link for"] version: Option<VersionChoice>,
) -> Result {
    let version = version.unwrap_or_default();

    let project = {
        let mut p = crate::api::curseforge::get_project_info(&ctx, version.id()).await?;
        p.latest_files
            .sort_unstable_by_key(|f| std::cmp::Reverse(f.file_date));
        p
    };

    if project.id != version.id() {
        println!("=== ERROR ===\nCurseforge API call returned the wrong project\n=== END ===");
        return Ok(());
    }

    let file = if let Some(file) = project.latest_files.get(0) {
        file
    } else {
        println!("=== ERROR ===\nNo Curseforge latest file\n=== END ===");
        return Ok(());
    };

    let url = format!(
        "{}/files/{}",
        project.links.website_url.trim_end_matches('/'),
        file.id
    );

    let mod_version = format!(
        "Download {:?} {}",
        version,
        file.file_name
            .rfind(&[' ', '-', '_', '+', 'v'][..])
            .map(|i| file.file_name[i + 1..].trim_end_matches(".jar"))
            .unwrap_or_default()
    );

    ctx.send(|m| {
        m.embed(|e| {
            e.thumbnail(&project.logo.thumbnail_url);
            e.author(|a| {
                a.name("Curseforge")
                    .icon_url(crate::constants::curseforge::ICON)
            })
            .colour(crate::constants::curseforge::COLOUR)
            .title(&project.name)
            .url(&project.links.website_url)
            .description(&project.summary)
            .field(
                "Latest Version",
                format!(
                    "[{}]({}) ({})",
                    file.file_name,
                    url,
                    crate::utils::pretty_bytesize(file.file_length)
                ),
                false,
            )
            .footer(|f| {
                f.text(format!(
                    "Total download count: {}",
                    crate::utils::pretty_large_int(project.download_count as u64)
                ))
            })
            .timestamp(file.file_date)
        })
        .components(|c| {
            c.create_action_row(|a| {
                a.create_button(|b| b.style(ButtonStyle::Link).label(mod_version).url(&url))
            })
        })
    })
    .await
    .unwrap();
    Ok(())
}
