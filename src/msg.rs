use std::path::{Path, PathBuf};
use std::collections::BTreeMap;
use failure::{err_msg, Error};

#[derive(Clone, Eq, PartialEq)]
pub enum Msg {
    Text(String),
    Compound(Vec<Msg>),
    Ext {
        name: String,
        params: BTreeMap<String, String>,
    },
}
impl Msg {
    /// Rebuild message with destructed format string and non-text segments.
    /// Where the char `\`` occurs will be inserted with an message in order.
    /// "\`\`" will be escaped as a single `\`` combined in text.
    pub fn construct(fmt: &str, msgs: &[Msg]) -> Result<Msg, Error> {
        if fmt.len() == 0 {
            return Ok(Msg::Text(String::new()))
        }
        // Initialize buffers with some reasonable values.
        let mut rv = Vec::<Msg>::with_capacity(5);
        let mut sb = String::with_capacity(fmt.len() / 2);
        let mut msg_index = 0;
        let mut left_pos = None;
        let mut right_pos = None;
        for (i, c) in fmt.chars().enumerate() {
            match c {
                '{' => {
                    if right_pos.is_some() {
                        // Expecting a "}}" escape, but there is only one '}',
                        // which is not allowed.
                        return Err(err_msg("Invalid format stirng."));
                    }
                    if (left_pos.is_some() && left_pos.unwrap() == i - 1) {
                        // Escape "{{".
                        sb.push('{');
                        left_pos = None;
                    } else {
                        // Found '{' and expect a '}' to close the .
                        left_pos = Some(i);
                    }
                },
                '}' => {
                    if left_pos.is_some() {
                        // '{' found, and here is a '}'! We can insert something
                        // now.
                        left_pos = None;
                        let msg = &msgs[msg_index];
                        if let Msg::Text(ref content) = msg {
                            // What? Why you insert text among lots of text?
                            sb.push_str(&content.clone());
                        } else {
                            if (sb.len() > 0) {
                                rv.push(Msg::Text(sb.clone()));
                                sb.clear();
                            }
                            if let Msg::Compound(ref segs) = msg {
                                // Insert all segments in a compound.
                                for seg in segs {
                                    rv.push(seg.clone());
                                }
                            } else {
                                // Insert a serious message.
                                rv.push(msg.clone());
                            }
                        }
                        msg_index = 0;
                    } else {
                        if (right_pos.is_some() &&
                                right_pos.unwrap() == i - 1) {
                            // Escape "}}".
                            sb.push('}');
                            right_pos = None;
                        } else {
                            // Found '}' and expect the next also a '}' as an
                            // escape.
                            right_pos = Some(i);
                        }
                    }
                },
                c => {
                    if right_pos.is_some() {
                        // Expecting a "}}" escape, but there is only one '}',
                        // which is not allowed.
                        return Err(err_msg("unclosed escape"))
                    } else if left_pos.is_some() {
                        // It's part of an index. Ensure it's a digit and don't
                        // push to the output buffer.
                        if let Some(d) = c.to_digit(10) {
                            // Good index!
                            msg_index = msg_index * 10 + d as usize;
                        } else {
                            return Err(err_msg("non-digit index"))
                        }
                    } else {
                        // Just plain text, push to buffer.
                        sb.push(c);
                    }
                },
            }
        }
        if left_pos.is_some() {
            return Err(err_msg("insertion indicator not closed."))
        } else if right_pos.is_some() {
            return Err(err_msg("unclosed escape"))
        } else if sb.len() > 0 {
            // Push remaining text.
            rv.push(Msg::Text(sb));
        }
        if (rv.len() > 1) {
            Ok(Msg::Compound(rv))
        } else {
            Ok(rv.pop().unwrap())
        }
    }
    fn destruct_impl(&self,
                     msg_count: &mut usize,
                     str_out: &mut String,
                     msg_out: &mut Vec<Msg>) {
        match self {
            Msg::Text(ref content) => {
                str_out.reserve(content.len());
                for c in content.chars() {
                    match c {
                        '{' => {
                            str_out.push('{');
                            str_out.push('{');
                        },
                        c => str_out.push(c),
                        '}' => {
                            str_out.push('}');
                            str_out.push('}');
                        },
                    }
                }
                str_out.push_str(content);
            },
            Msg::Compound(ref segs) => {
                for seg in segs {
                    seg.destruct_impl(msg_count, str_out, msg_out);
                }
            },
            ext => {
                str_out.push('{');
                str_out.push_str(&msg_count.to_string());
                str_out.push('}');
                *msg_count += 1;
            },
        }
    }
    pub fn destruct(&self) -> (String, Vec<Msg>) {
        match self {
            Msg::Text(content) => (content.clone(), Vec::new()),
            Msg::Compound(segs) => {
                let mut count = 0;
                let mut fmt = String::new();
                let mut msgs = Vec::new();
                for seg in segs {
                    seg.destruct_impl(&mut count, &mut fmt, &mut msgs);
                }
                (fmt, msgs)
            },
            _ => ("{0}".to_owned(), vec![ self.clone() ]),
        }
    }
    pub fn text(text: &str) -> Msg {
        Msg::Text(text.to_string())
    }
    pub fn at(qq: i64) -> Msg {
        ExtBuilder::new("at")
            .with_param("qq", &qq.to_string())
            .build()
    }
    pub fn image(path: &Path) -> Msg {
        ExtBuilder::new("image")
            .with_param("file", &path.to_string_lossy())
            .build()
    }
    pub fn record(path: &Path) -> Msg {
        ExtBuilder::new("record")
            .with_param("file", &path.to_string_lossy())
            .build()
    }
}
impl<T> From<T> for Msg where T: 'static  + AsRef<str> {
    fn from(x: T) -> Msg {
        Msg::Text(x.as_ref().to_owned())
    }
}

macro_rules! msg {
    ( $( $msg: expr ),* ) => {
        let mut rv = Vec::new();
        $( rv.push($msg); )*
        Msg::Compound(rv)
    }
}

pub struct ExtBuilder {
    name: String,
    params: BTreeMap<String, String>,
}
impl ExtBuilder {
    pub fn new(name: &str) -> ExtBuilder {
        ExtBuilder {
            name: name.to_owned(),
            params: BTreeMap::new(),
        }
    }
    pub fn with_param(mut self, key: &str, value: &str) -> Self {
        self.params
            .entry(key.to_owned())
            .or_insert(value.to_owned());
        self
    }
    pub fn build(self) -> Msg {
        Msg::Ext {
            name: self.name,
            params: self.params,
        }
    }
}

pub enum MsgIn {
    Private {
        qq: i64,
        alias: String,
        content: String,
    },
    Group {
        grp: i64,
        qq: i64,
        alias: String,
        grp_alias: String,
        content: String,
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_construct() {
    }
}
