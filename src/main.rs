#[macro_use]
extern crate serde_json;
extern crate json_patch;

use json_patch::patch;
use serde_json::from_str;

fn main() {
    let mut doc = json!([
        { "name": "Andrew" },
        { "name": "Maxim" }
    ]);

    let p = from_str(
        r#"[
  { "op": "test", "path": "/0/name", "value": "Andrew" },
  { "op": "add", "path": "/0/happy", "value": true }
]"#,
    )
    .unwrap();

    patch(&mut doc, &p).unwrap();
    assert_eq!(
        doc,
        json!([
          { "name": "Andrew", "happy": true },
          { "name": "Maxim" }
        ])
    );
}
