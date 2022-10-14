use molt::{check_args, molt_ok, ContextID, Interp, MoltResult, Value};

use super::tcl::DiscordContext;

pub(crate) fn cmd_output(interp: &mut Interp, _: ContextID, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 1, 0, "string")?;

    let res = argv.iter().map(|arg| arg.as_str()).collect::<Vec<&str>>()[1..].join(" ");
    interp.push_output(&res);
    molt_ok!()
}

pub(crate) fn cmd_proc_names(interp: &mut Interp, _: ContextID, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 1, 2, "string")?;

    let proc_names = interp.proc_names();

    let mut names: Vec<&str> = proc_names.iter().map(|val| val.as_str()).collect();

    if argv.len() > 1 {
        names = names
            .into_iter()
            .filter_map(|proc| match proc.contains(argv[1].as_str()) {
                true => Some(proc),
                false => None,
            })
            .collect();
    }

    molt_ok!(Value::from(names.join(" ")))
}

pub(crate) fn cmd_show_proc(interp: &mut Interp, _: ContextID, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 2, 2, "string")?;

    let args = interp.proc_args(argv[1].as_str()).unwrap();
    let body = interp.proc_body(argv[1].as_str()).unwrap();

    interp.push_output(&format!("proc {} {{{}}} {{{}}}", argv[1], args, body));

    molt_ok!()
}

pub(crate) fn cmd_rand(interp: &mut Interp, _: ContextID, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 1, 1, "")?;
    molt_ok!(interp.get_rand())
}

pub(crate) fn cmd_me(interp: &mut Interp, context_id: ContextID, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 1, 1, "")?;
    let id = interp.context::<DiscordContext>(context_id).me.unwrap().0;
    molt_ok!(Value::from(format!("<@{}>", id)))
}

pub(crate) fn cmd_channel(
    interp: &mut Interp,
    context_id: ContextID,
    argv: &[Value],
) -> MoltResult {
    check_args(1, argv, 1, 1, "")?;
    let id = interp
        .context::<DiscordContext>(context_id)
        .channel
        .unwrap()
        .0;
    molt_ok!(Value::from(format!("<#{}>", id)))
}

pub(crate) fn cmd_users(interp: &mut Interp, context_id: ContextID, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 1, 1, "")?;
    let users = interp
        .context::<DiscordContext>(context_id)
        .users
        .as_ref()
        .unwrap()
        .iter()
        .map(|uid| Value::from(format!("<@{}>", uid)))
        .collect::<Vec<Value>>();

    molt_ok!(Value::from(users))
}

pub(crate) fn cmd_int(_interp: &mut Interp, _context_id: ContextID, argv: &[Value]) -> MoltResult {
    check_args(1, argv, 2, 2, "value")?;

    let val = argv[1].as_float()?;

    molt_ok!(Value::from(val as i64))
}
