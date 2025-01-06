use serenity::model::channel::Reaction;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::database::qa_data::*;

fn extract_image_attachment(msg: &Message) -> Option<&Attachment> {
    msg.attachments.first().and_then(|a| {
        if a.content_type
            .as_ref()
            .map(|s| s.starts_with("image"))
            .unwrap_or_default()
        {
            Some(a)
        } else {
            None
        }
    })
}

pub async fn handle_reaction(ctx: &Context, reaction: Reaction, guild_id: GuildId) {
    if !is_questions_channel(ctx, guild_id, reaction.channel_id)
        .await
        .unwrap_or_default()
    {
        return;
    }

    let user_id = reaction
        .user_id
        .expect("Could not find user_id in reaction...");

    if user_id == crate::constants::BOT_ID
        || !is_qa_moderator(ctx, user_id, guild_id)
            .await
            .unwrap_or_default()
    {
        return;
    }

    let Some(answers_channel) = get_answer_channel(ctx, guild_id).await else {
        println!("Could not find answer channel for this server");
        return;
    };

    let Ok(answer) = reaction.message(ctx).await else {
        println!("Could not get answer message from its id");
        return;
    };

    if answer.author.id != user_id
        || !answer
            .reactions
            .iter()
            .any(|react| react.me && react.reaction_type.unicode_eq("❓"))
    {
        return;
    } else if let Err(e) = answer.delete_reaction_emoji(ctx, '❓').await {
        println!("Error deleting '❓' reaction: {e}");
    }

    let Some(ref question) = answer.referenced_message else {
        println!("Answer has no referenced question message");
        return;
    };

    let answer_attachment = extract_image_attachment(&answer);
    let question_attachment = extract_image_attachment(question);

    if let Err(e) = answers_channel
        .send_message(ctx, |m| {
            if let Some(attachment) = question_attachment {
                m.add_file(attachment.url.as_str());
            }
            if let Some(attachment) = answer_attachment {
                m.add_file(attachment.url.as_str());
            }
            m.embed(|e| {
                if let Some(attachment) = question_attachment {
                    e.thumbnail(format!("attachment://{}", attachment.filename));
                }
                if answer.content.len() <= 1024 && question.content.len() <= 1024 {
                    e.fields([
                        (
                            format!("**Question by {}**", question.author.name),
                            &question.content,
                            false,
                        ),
                        (
                            format!("**Answer by {}**", answer.author.name),
                            &answer.content,
                            false,
                        ),
                    ]);
                } else {
                    e.description(format!(
                        "**Question by {question_author}**
{question_content}

**Answer by {answer_author}**
{answer_content}",
                        question_author = question.author.name,
                        question_content = question.content,
                        answer_author = answer.author.name,
                        answer_content = answer.content
                    ));
                }
                e.author(|a| a.name("LOTR Mod Q&A").icon_url(crate::constants::BOT_ICON));
                e.colour(0xc27c0e);
                if let Some(attachment) = answer_attachment {
                    e.attachment(&attachment.filename);
                }
                e.footer(|f| f.icon_url(answer.author.face()));
                e.timestamp(answer.timestamp);
                e
            })
        })
        .await
    {
        println!("=== Q&A ERROR ===\nError sending Q&A answer: {e}\n=== END ===");
    }
}

pub async fn handle_message(ctx: &Context, message: &Message, guild_id: GuildId) {
    if !is_questions_channel(ctx, guild_id, message.channel_id)
        .await
        .unwrap_or_default()
    {
        return;
    }

    if !is_qa_moderator(ctx, message.author.id, guild_id)
        .await
        .unwrap_or_default()
    {
        return;
    }

    if let Err(e) = message.react(ctx, '❓').await {
        println!("Could not add reaction to message: {e}");
    }
}
