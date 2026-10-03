use std::collections::HashMap;
pub struct HttpResponse{
    pub status_code: u16,
    pub headers: HashMap<String,String>,
    pub body: String,
}