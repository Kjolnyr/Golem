use std::collections::{HashMap, HashSet};
use std::sync::Arc;

use serenity::async_trait;
use serenity::framework::standard::macros::{group, hook};
use serenity::framework::standard::StandardFramework;
use serenity::model::channel::Message;
use serenity::model::prelude::{ChannelId, GuildId, Ready, UserId};
use serenity::model::Permissions;
use serenity::prelude::*;

use crate::config::Config;
use crate::tcl::tcl::Tcl;

struct Data {
    chan_users: HashMap<ChannelId, Vec<UserId>>,
}

impl Default for Data {
    fn default() -> Self {
        Self {
            chan_users: Default::default(),
        }
    }
}

impl Data {
    pub async fn update(&mut self, ctx: &Context) {
        let guilds = ctx.http.get_guilds(None, None).await.unwrap();
        for guild_info in guilds {
            let guild = guild_info.id.to_partial_guild(&ctx.http).await.unwrap();

            let channels = guild.channels(&ctx.http).await.unwrap();
            let members = guild.members(&ctx.http, None, None).await.unwrap();

            for (channel_id, channel) in &channels {
                let mut ok_members = vec![];
                for member in &members {
                    let perms = guild.user_permissions_in(channel, member).unwrap();
                    if perms.contains(Permissions::VIEW_CHANNEL) {
                        ok_members.push(member.user.id);
                    }
                }
                self.chan_users.insert(*channel_id, ok_members);
            }
        }
    }
}

struct DataContainer;

impl TypeMapKey for DataContainer {
    type Value = Arc<RwLock<Data>>;
}

struct TclContainer;

impl TypeMapKey for TclContainer {
    type Value = Arc<RwLock<Tcl>>;
}

struct ConfigContainer;

impl TypeMapKey for ConfigContainer {
    type Value = Arc<Config>;
}

#[group]
struct General;

#[group]
#[owners_only]
struct Owner;

struct Handler;

#[hook]
async fn handle_command(ctx: &Context, msg: &Message, _command_name: &str) {
    let args = msg.content.split(' ').collect::<Vec<&str>>();

    let d = ctx.data.read().await;

    let tcl_arc = d.get::<TclContainer>().unwrap();
    let data_arc = d.get::<DataContainer>().unwrap();
    let config = d.get::<ConfigContainer>().unwrap();

    if !config.bot_owners.contains(msg.author.id.as_u64()) {
        return;
    }

    let data = data_arc.write().await;
    let mut tcl = tcl_arc.write().await;

    let empty: Vec<UserId> = vec![];
    let users = data.chan_users.get(&msg.channel_id).unwrap_or(&empty);

    let discord_ctx = tcl.get_interp_context_mut();

    discord_ctx.me = Some(msg.author.id);
    discord_ctx.channel = Some(msg.channel_id);
    discord_ctx.users = Some(users.clone());

    let result = tcl.run(&args[1..]);

    if result.len() > 0 {
        msg.channel_id.say(&ctx.http, result).await.unwrap();
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn ready(&self, ctx: Context, bot_data: Ready) {
        let mut user = bot_data.user;

        let d = ctx.data.read().await;
        let config = d.get::<ConfigContainer>().unwrap();

        if config.bot_name != user.name {
            user.edit(&ctx.http, |p| p.username(&config.bot_name))
                .await
                .unwrap();
        }
    }

    async fn cache_ready(&self, ctx: Context, _guilds: Vec<GuildId>) {
        let d = ctx.data.read().await;
        let data_arc = d.get::<DataContainer>().unwrap();

        let mut data = data_arc.write().await;

        data.update(&ctx).await;

        println!("Ready!");
    }
}

#[tokio::main]
pub async fn discord_init(config_path: &str) {
    let config = match Config::load(config_path) {
        Ok(config) => config,
        Err(e) => panic!("Error: {:?}", e),
    };

    let mut owners: HashSet<UserId> = HashSet::new();

    config.bot_owners.iter().for_each(|owner| {
        owners.insert(UserId(*owner));
    });

    let framework = StandardFramework::new()
        .configure(|c| c.prefix("tcl ").owners(owners))
        .group(&GENERAL_GROUP)
        .group(&OWNER_GROUP)
        .unrecognised_command(handle_command);

    // Login with a bot token from the environment
    let intents = GatewayIntents::non_privileged()
        | GatewayIntents::MESSAGE_CONTENT
        | GatewayIntents::DIRECT_MESSAGES
        | GatewayIntents::GUILDS
        | GatewayIntents::GUILD_MEMBERS;
    let mut client = Client::builder(config.bot_token.clone(), intents)
        .event_handler(Handler)
        .framework(framework)
        .type_map_insert::<TclContainer>(Arc::new(RwLock::new(Tcl::new(
            &config.proc_path,
            &config.vars_path,
        ))))
        .type_map_insert::<DataContainer>(Arc::new(RwLock::new(Data::default())))
        .type_map_insert::<ConfigContainer>(Arc::new(config.clone()))
        .await
        .expect("Error creating client");

    // start listening for events by starting a single shard
    if let Err(why) = client.start().await {
        println!("An error occurred while running the client: {:?}", why);
    }
}
