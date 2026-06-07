use serenity::all::{
    ChannelId, CreateAllowedMentions, CreateMessage, GuildId, RoleId, User, UserId,
};
use serenity::async_trait;
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::prelude::*;

use crate::{Action, Config};

pub struct Handler {
    channel_watchlist: Vec<ChannelId>,
    user_whitelist: Vec<UserId>,
    role_whitelist: Vec<RoleId>,
    logging_channel: Option<ChannelId>,
    admin_role: Option<RoleId>,
    action: Action,
    dmd: u8,
}

impl Handler {
    pub fn from_config(config: &Config) -> Handler {
        Handler {
            channel_watchlist: config
                .channel_watchlist
                .iter()
                .map(|s| {
                    s.parse().expect(&format!(
                        "Could not get channel ID (for watchlist) from '{s}'"
                    ))
                })
                .collect(),
            user_whitelist: {
                if let Some(first) = config.user_whitelist.first()
                    && !first.is_empty()
                {
                    config
                        .user_whitelist
                        .iter()
                        .map(|s| {
                            s.parse().expect(&format!(
                                "Could not get user ID (for whitelist) from '{s}'"
                            ))
                        })
                        .collect()
                } else {
                    vec![]
                }
            },
            role_whitelist: {
                if let Some(first) = config.role_whitelist.first()
                    && !first.is_empty()
                {
                    config
                        .role_whitelist
                        .iter()
                        .map(|s| {
                            s.parse().expect(&format!(
                                "Could not get role ID (for whitelist) from '{s}'"
                            ))
                        })
                        .collect()
                } else {
                    vec![]
                }
            },
            logging_channel: if config.logging_channel.is_empty() {
                None
            } else {
                Some(config.logging_channel.parse().expect(&format!(
                    "Could not get channel ID (for logging) from '{}'",
                    config.logging_channel
                )))
            },
            admin_role: if config.admin_role.is_empty() {
                None
            } else {
                Some(config.admin_role.parse().expect(&format!(
                    "Could not get admin role ID (for logging) from '{}'",
                    config.admin_role
                )))
            },
            action: config.action.clone(),
            dmd: config.dmd,
        }
    }

    async fn author_has_role(
        &self,
        ctx: &Context,
        msg: &Message,
        guild_id: &GuildId,
        role_id: &RoleId,
    ) -> bool {
        msg.author
            .has_role(&ctx, guild_id, role_id)
            .await
            .expect("Unable to determine whether user has role")
    }

    async fn should_allow_message(
        &self,
        ctx: &Context,
        msg: &Message,
        user_id: &UserId,
        guild_id: &GuildId,
    ) -> bool {
        // Allowed because the bot sent this message and we don't want recursion
        if user_id == &ctx.cache.current_user().id {
            return true;
        }

        // Allowed because this is not a watched channel
        if !self.channel_watchlist.contains(&msg.channel_id) {
            return true;
        }

        // Allowed because user is whitelisted by ID
        if self.user_whitelist.contains(&user_id) {
            return true;
        }

        // Allowed because user is whitelisted by role
        for role in self.role_whitelist.iter() {
            {
                if self.author_has_role(&ctx, &msg, &guild_id, &role).await {
                    return true;
                }
            }
        }

        // Allowed because user is an admin
        if let Some(admin_role) = self.admin_role {
            if self
                .author_has_role(&ctx, &msg, &guild_id, &admin_role)
                .await
            {
                return true;
            }
        }

        return false;
    }

    async fn notify(
        &self,
        ctx: &Context,
        msg: &Message,
        channel_id: &ChannelId,
        author: &User,
        action: &Action,
    ) {
        if action == &Action::LogOnly {
            let mut content = if let Some(role) = self.admin_role {
                format!("{}, ", role.mention())
            } else {
                "".to_string()
            };
            content.push_str(&format!(
                "{} has posted in {}",
                msg.author.mention(),
                msg.channel_id.mention(),
            ));

            channel_id
                .send_message(
                    &ctx,
                    CreateMessage::new().content(content).allowed_mentions(
                        CreateAllowedMentions::new()
                            .roles(self.admin_role.map_or(vec![], |role| vec![role]))
                            .users(vec![msg.author.id]),
                    ),
                )
                .await
                .expect(&format!("Error sending log message"));
            return;
        } else {
            channel_id
                .send_message(
                    &ctx,
                    CreateMessage::new().content(&format!(
                        "User {} ({}) was {} for falling into the honeypot.",
                        author.display_name(),
                        author.id,
                        match action {
                            Action::Kick => "kicked ",
                            Action::Ban => "banned ",
                            Action::LogOnly => unreachable!(),
                        }
                    )),
                )
                .await
                .expect(&format!("Error sending log message"));
        }
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn message(&self, ctx: Context, msg: Message) {
        let user_id = msg.author.id;
        let Some(guild_id) = msg.guild_id else {
            // Somehow invoked outside of a server
            return;
        };

        if self
            .should_allow_message(&ctx, &msg, &user_id, &guild_id)
            .await
        {
            return;
        }

        // Notification to log channel (non-actionable, just informative)
        if let Some(logging_channel) = self.logging_channel {
            self.notify(&ctx, &msg, &logging_channel, &msg.author, &self.action)
                .await;
        }

        if let Some(guild_id) = msg.guild_id {
            msg.delete(&ctx).await.expect("Unable to delete message");
            match self.action {
                // Kick the user
                Action::Kick => {
                    guild_id
                        .kick_with_reason(&ctx, user_id, "Fell into the honeypot")
                        .await
                        .expect(&format!(
                            "Could not kick user {} ({})",
                            msg.author.display_name(),
                            user_id
                        ));
                }
                // Ban the user & delete their messages from the past `self.dmd` days
                Action::Ban => {
                    guild_id
                        .ban_with_reason(&ctx, user_id, self.dmd, "Fell into the honeypot")
                        .await
                        .expect(&format!(
                            "Could not ban user {} ({})",
                            msg.author.display_name(),
                            user_id
                        ));
                }
                // Ping authority in log channel (if you wanted manual intervention)
                Action::LogOnly => {
                    if self.logging_channel.is_none() {
                        eprintln!(
                            "Action set to log-only but no logging channel ID was set. Doing nothing."
                        );
                    }
                }
            }
        }
    }

    async fn ready(&self, _: Context, ready: Ready) {
        println!("{} is on patrol...", ready.user.name);
    }
}
