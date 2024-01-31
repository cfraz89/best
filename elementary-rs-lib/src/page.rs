use crate::node::Node;

pub struct Page(pub Node);

impl From<Node> for Page {
    fn from(node: Node) -> Self {
        Self(node)
    }
}

impl Page {
    pub fn render(&self, wasm_path: &str) -> String {
        format!(
            "<!doctype html><html><body>{}<script type=\"module\">import start from \"{}\"; await start();</script></body></html>",
            self.0, wasm_path
        )
    }
}
