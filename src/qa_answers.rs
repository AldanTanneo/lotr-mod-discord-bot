use serenity::model::channel::Reaction;
use serenity::model::prelude::*;
use serenity::prelude::*;

use crate::database::qa_data::*;

fn extract_image_attachment(msg: &Message) -> Option<&Attachment> {
    msg.attachments
        .get(0)
        .map(|a| {
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
        .flatten()
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

    if let Some(answers_channel) = get_answer_channel(ctx, guild_id).await {
        if let Ok(ref answer) = reaction.message(ctx).await {
            if answer.author.id != user_id
                || !answer
                    .reactions
                    .iter()
                    .any(|react| react.me && react.reaction_type.unicode_eq("❓"))
            {
                return;
            } else if let Err(e) = answer.delete_reaction_emoji(ctx, '❓').await {
                println!("Error deleting '❓' reaction: {}", e);
            }

            if let Some(ref question) = answer.referenced_message {
                let answer_attachment = extract_image_attachment(answer);
                let question_attachment = extract_image_attachment(question);

                answers_channel
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
                            e.author(|a| {
                                a.name("LOTR Mod Q&A").icon_url(crate::constants::BOT_ICON)
                            });
                            e.colour(0xc27c0e);
                            if let Some(attachment) = answer_attachment {
                                e.attachment(&attachment.filename);
                            }
                            e.footer(|f| f.icon_url(answer.author.face()));
                            e.timestamp(&answer.timestamp);
                            e
                        })
                    })
                    .await
                    .expect("Could not send answer message in answers channel");
            } else {
                println!("Answer has no referenced question message");
            }
        } else {
            println!("Could not get answer message from its id");
        }
    } else {
        println!("Could not find answer channel for this server");
    }
}

pub async fn handle_message(ctx: &Context, message: &Message, guild_id: GuildId) {
    if message.referenced_message.is_none() {
        return;
    }

    if !is_questions_channel(ctx, guild_id, message.channel_id)
        .await
        .unwrap_or_default()
    {
        println!("Not a questions channel");
        return;
    }

    if !is_qa_moderator(ctx, message.author.id, guild_id)
        .await
        .unwrap_or_default()
    {
        println!("Not a q&a mod");
        return;
    }

    if let Err(e) = message.react(ctx, '❓').await {
        println!("Could not add reaction to message: {}", e);
    }
}
