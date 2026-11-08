//! Sementara: dump semua kunci registry (nama + alias) untuk audit paritas.
use std::collections::BTreeMap;
use std::io::Write;

// Memaksa crate plugin di-link agar `linkme::distributed_slice` dari
// `#[plugin]` ikut terkumpul ke registry.
extern crate oktz_plugins;

fn main() {
    let mut map: BTreeMap<String, Vec<String>> = BTreeMap::new();
    for p in oktz_plugin_api::all_plugins() {
        let mut k = vec![p.meta.name.clone()];
        k.extend(p.meta.alias.iter().cloned());
        k.extend(p.meta.commands.iter().cloned());
        for key in k {
            map.entry(key).or_default().push(p.meta.name.clone());
        }
    }
    let mut out = std::io::stdout();
    for (key, owners) in &map {
        let _ = writeln!(out, "{key}\t{}", owners.join(","));
    }
}
