use anyhow::{Context, Result};
use std::collections::HashMap;

pub enum Method {
    GET,
    POST,
}

impl TryFrom<&str> for Method {
    type Error = anyhow::Error;

    fn try_from(value: &str) -> Result<Self, anyhow::Error> {
        match value {
            "GET" => Ok(Method::GET),
            "POST" => Ok(Method::POST),
            _ => Err(anyhow::anyhow!("Method not supported")),
        }
    }
}

pub struct Request {
    pub method: Method,
    pub path: String,
    pub headers: std::collections::HashMap<String, String>,
    pub body: String,
}

impl Request {
    pub async fn new(request: std::borrow::Cow<'_, str>) -> Result<Self> {
        let mut parts = request.split("\r\n\r\n");
        let head = parts.next().context("Headline Error")?;
        // Body
        let body = parts.next().unwrap_or("");

        // Method and path
        let mut head_line = head.lines();
        let first = head_line.next().context("Empty Request")?;
        let mut request_parts: std::str::SplitWhitespace<'_> = first.split_whitespace();
        let method: Method = request_parts
            .next()
            .ok_or(anyhow::anyhow!("missing method"))
            .and_then(TryInto::try_into)
            .context("Missing Method")?;
        let path = request_parts.next().context("No Path")?;

        // Headers
        let mut headers = HashMap::new();
        for line in head_line {
            if let Some((k, v)) = line.split_once(":") {
                headers.insert(k.trim().to_lowercase(), v.trim().to_string());
            }
        }
        Ok(Request {
            method,
            path: path.into(),
            headers,
            body: body.into(),
        })
    }
}
