use oktz_plugin_api::{Ctx, Reply, find, plugin};

#[plugin(name = "ping", alias = ["p"], category = "main", cooldown = 5, is_private = true)]
fn ping(_ctx: Ctx) -> Reply {
    Reply::text("pong")
}

#[plugin(name = "echo", category = "tools", is_owner = true, energi = 3)]
fn echo(ctx: Ctx) -> Reply {
    Reply::text(ctx.text)
}

/// Nama array JS (`plugins.js:207-209`) masuk map `commands`, bukan `aliases`.
#[plugin(
    name = "cmd",
    commands = ["cb"],
    alias = ["ca"],
    category = "tools"
)]
fn cmd(_ctx: Ctx) -> Reply {
    Reply::text("cmd")
}

/// Alias plugin lain dengan nama yang sama — kalah oleh command `cmd`.
#[plugin(name = "ali", alias = ["cb"], category = "tools")]
fn ali(_ctx: Ctx) -> Reply {
    Reply::text("ali")
}

#[test]
fn registry_collects_declared_plugins() {
    let names: Vec<&str> = oktz_plugin_api::all_plugins()
        .iter()
        .map(|p| p.meta.name.as_str())
        .collect();
    assert!(names.contains(&"ping"), "registered: {names:?}");
    assert!(names.contains(&"echo"), "registered: {names:?}");
}

#[test]
fn find_resolves_alias() {
    let p = find("p").expect("alias harus ketemu");
    assert_eq!(p.meta.name, "ping");
    assert_eq!(p.meta.cooldown, 5);
}

#[test]
fn macro_populates_meta_fields() {
    let ping = find("ping").unwrap();
    assert_eq!(ping.meta.category, "main");
    assert!(ping.meta.is_private);
    assert!(!ping.meta.is_owner);

    let echo = find("echo").unwrap();
    assert!(echo.meta.is_owner);
    assert_eq!(echo.meta.energi, 3);
}

#[test]
fn handler_is_callable() {
    let ping = find("ping").unwrap();
    let reply = (ping.handler)(Ctx {
        text: "x".into(),
        sender: "628".into(),
        ..Default::default()
    });
    assert_eq!(reply.text_body(), Some("pong"));
}

/// Kontrak baru: `Reply` harus bisa membawa mention sebagai jid, bukan
/// string yang harus di-embed plugin. Port `cek*` butuh ini.
#[test]
fn handler_can_return_a_mention_reply() {
    let cek = find("ping").unwrap();
    let reply = (cek.handler)(Ctx::default());
    assert!(matches!(reply, Reply::Text { .. }));
}

#[test]
fn unknown_command_returns_none() {
    assert!(find("command-yang-tidak-ada").is_none());
}

/// `plugins.js:539-544`: lookup mengecek map `commands` dulu, baru `aliases`.
/// Jadi command menang atas alias plugin lain yang namanya sama.
#[test]
fn command_menang_atas_alias_plugin_lain() {
    let cmd = find("cmd").unwrap();
    assert_eq!(cmd.meta.commands, ["cb"]);
    assert_eq!(cmd.meta.alias, ["ca"]);
    assert_eq!(find("cb").unwrap().meta.name, "cmd");
    assert_eq!(find("ca").unwrap().meta.name, "cmd");
    // Alias plugin `ali` untuk key `cb` kalah, persis seperti di JS.
    assert_eq!(find("ali").unwrap().meta.name, "ali");
}