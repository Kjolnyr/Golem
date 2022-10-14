use molt::{ContextID, Interp};
use serenity::model::prelude::{ChannelId, UserId};

use super::commands::*;

pub(crate) struct DiscordContext {
    pub(crate) me: Option<UserId>,
    pub(crate) channel: Option<ChannelId>,
    pub(crate) users: Option<Vec<UserId>>,
}

impl Default for DiscordContext {
    fn default() -> Self {
        Self {
            me: Default::default(),
            channel: Default::default(),
            users: Default::default(),
        }
    }
}

pub(crate) struct Tcl {
    interp: Interp,
    ctx_id: ContextID,
    proc_path: String,
    vars_path: String,
}

impl Tcl {
    pub(crate) fn new(proc_path: &str, vars_path: &str) -> Self {
        let mut interp = Interp::new();

        interp.load_vars(vars_path);
        interp.load_procs(proc_path);

        let ctx_id = interp.save_context(DiscordContext::default());
        Self::add_commands(&mut interp, ctx_id);

        Self {
            interp,
            ctx_id,
            proc_path: proc_path.to_string(),
            vars_path: vars_path.to_string(),
        }
    }

    pub(crate) fn get_interp_context_mut(&mut self) -> &mut DiscordContext {
        self.interp.context::<DiscordContext>(self.ctx_id)
    }

    fn add_commands(interp: &mut Interp, ctx_id: ContextID) {
        interp.add_command(".", cmd_output);
        interp.add_command("procs", cmd_proc_names);
        interp.add_command("show", cmd_show_proc);
        interp.add_command("rand", cmd_rand);
        interp.add_command("int", cmd_int);

        interp.add_context_command("me", cmd_me, ctx_id);
        interp.add_context_command("channel", cmd_channel, ctx_id);
        interp.add_context_command("users", cmd_users, ctx_id);
    }

    pub(crate) fn run(&mut self, args: &[&str]) -> String {
        let result = match self.interp.eval(&args.join(" ")) {
            Ok(res) => res,
            Err(e) => e.value(),
        };

        self.interp.save_vars(&self.vars_path);
        self.interp.save_procs(&self.proc_path);

        format!("{}{}", self.interp.get_output(), result)
    }
}
