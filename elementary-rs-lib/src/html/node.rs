#[derive(Debug)]
pub enum Node {
    Text(String),
    Element(Element),
}

#[derive(Debug)]
pub struct Element {
    pub tag: String,
    pub attributes: Vec<(String, String)>,
}
