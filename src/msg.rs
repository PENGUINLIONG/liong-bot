use std::path::{Path, PathBuf};
use std::collections::BTreeMap;
use failure::{err_msg, Error};

#[derive(Clone, Debug, Eq, PartialEq)]
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
                        '}' => {
                            str_out.push('}');
                            str_out.push('}');
                        },
                        c => str_out.push(c),
                    }
                }
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
                msg_out.push(ext.clone());
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
    pub fn starts_with<P>(&self, pat: &P) -> bool
            where P: ?Sized + AsRef<str> {
        // FIXME: Use `Pattern<'a>` when it's stablized.
        match self {
            Msg::Text(ref content) => content.starts_with(pat.as_ref()),
            Msg::Compound(ref segs) => {
                if let Some(ref msg) = segs.get(0) {
                    msg.starts_with(pat)
                } else {
                    "".starts_with(pat.as_ref())
                }
            },
            _ => "".starts_with(pat.as_ref()),
        }
    }
}
impl<T> From<T> for Msg where T: 'static  + AsRef<str> {
    fn from(x: T) -> Msg {
        Msg::Text(x.as_ref().to_owned())
    }
}

pub struct ExtBuilder {
    name: String,
    params: BTreeMap<String, String>,
}
impl ExtBuilder {
    pub fn new<S>(name: &S) -> ExtBuilder
            where S: ?Sized + ToString {
        ExtBuilder {
            name: name.to_string(),
            params: BTreeMap::new(),
        }
    }
    pub fn add_param<K, V>(&mut self, key: &K, value: &V)
            where K: ?Sized + ToString, V: ?Sized + ToString {
        self.params
            .entry(key.to_string())
            .or_insert(value.to_string());
    }
    pub fn with_param<K, V>(mut self, key: &K, value: &V) -> Self
            where K: ?Sized + ToString, V: ?Sized + ToString {
        self.add_param(key, value);
        self
    }
    pub fn build(self) -> Msg {
        Msg::Ext {
            name: self.name,
            params: self.params,
        }
    }
}

pub fn empty() -> Msg {
    Msg::Text(String::new())
}
pub fn text(text: &str) -> Msg {
    Msg::Text(text.to_string())
}
pub fn at(qq: i64) -> Msg {
    ExtBuilder::new("at")
        .with_param("qq", &qq.to_string())
        .build()
}
pub fn image<P: ?Sized + AsRef<Path>>(path: &P) -> Msg {
    ExtBuilder::new("image")
        .with_param("file", &path.as_ref().to_string_lossy())
        .build()
}
pub fn record<P: ?Sized + AsRef<Path>>(path: &P) -> Msg {
    ExtBuilder::new("record")
        .with_param("file", &path.as_ref().to_string_lossy())
        .build()
}

pub struct MsgBuilder(Vec<Msg>);
impl MsgBuilder {
    pub fn new() -> MsgBuilder {
        MsgBuilder(Vec::new())
    }
    pub fn add_msg(&mut self, msg: Msg) {
        match msg {
            Msg::Text(content) => {
                if let Some((Msg::Text(last), _)) = self.0.split_last_mut() {
                    last.push_str(&content);
                } else {
                    self.0.push(Msg::Text(content));
                }
            },
            Msg::Compound(mut segs) => self.0.append(&mut segs),
            _ => self.0.push(msg),
        }
    }
    pub fn with_msg(mut self, msg: Msg) -> Self {
        self.add_msg(msg);
        self
    }
    pub fn build(mut self) -> Msg {
        match self.0.len() {
            0 => empty(),
            1 => self.0.pop().unwrap(),
            _ => Msg::Compound(self.0),
        }
    }
}

#[macro_export]
macro_rules! msg {
    ( $($msg: expr),* ) => {{
        let mut rv = MsgBuilder::new();
        $( rv = rv.with_msg(Msg::from($msg)); )*
        rv.build()
    }}
}

pub enum MsgIn {
    Private {
        qq: i64,
        alias: String,
        content: Msg,
    },
    Group {
        grp: i64,
        qq: i64,
        alias: String,
        grp_alias: String,
        content: Msg,
    }
}
impl MsgIn {
    pub fn is_priv(&self) -> bool {
        if let MsgIn::Private { .. } = self {
            true
        } else {
            false
        }
    }
    pub fn is_grp(&self) -> bool {
        if let MsgIn::Group { .. } = self {
            true
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_shortcut() {
        assert_eq!(
            text("123"),
            Msg::Text("123".to_string()),
        );
        assert_eq!(
            at(123),
            ExtBuilder::new("at")
                .with_param("qq", "123")
                .build(),
        );
        assert_eq!(
            image(&Path::new("./image.jpg")),
            ExtBuilder::new("image")
                .with_param("file", "./image.jpg")
                .build(),
        );
        assert_eq!(
            record(&Path::new("./record.silk")),
            ExtBuilder::new("record")
                .with_param("file", "./record.silk")
                .build(),
        );
    }
    #[test]
    fn test_construct() {
        let text_msg = text("123");
        let text_construct = Msg::construct("{0}", &vec![text_msg.clone()]);
        assert_eq!(text_msg, text_construct.unwrap());
        let ext_msg = at(123);
        let ext_construct = Msg::construct("{0}", &vec![ext_msg.clone()]);
        assert_eq!(ext_msg, ext_construct.unwrap());
        let cpd_msg = msg!["123123", ext_msg.clone()];
        let cpd_construct = Msg::construct("123{0}{1}",
                                           &vec![text_msg, ext_msg]);
        assert_eq!(cpd_msg, cpd_construct.unwrap());
    }
    #[test]
    fn test_destruct() {
        let (fmt, msgs) = msg!["123", at(123), "456", at(456)].destruct();
        assert_eq!(fmt, "123{0}456{1}");
        assert_eq!(msgs, vec![at(123), at(456)]);
        let (fmt, msgs) = at(123).destruct();
        assert_eq!(fmt, "{0}");
        assert_eq!(msgs, vec![at(123)]);
        let (fmt, msgs) = text("123").destruct();
        assert_eq!(fmt, "123");
        assert_eq!(msgs, Vec::new());
    }
    #[test]
    fn test_escape() {
        let (fmt, _) = msg!["{{}", at(123), "}}{"].destruct();
        assert_eq!(fmt, "{{{{}}{0}}}}}{{");

        let msg = Msg::construct("{{{0}}}", &vec![at(123)]);
        assert_eq!(msg.unwrap(), msg!["{", at(123), "}"]);
    }
    #[test]
    fn test_starts_with() {
        assert_eq!(empty().starts_with(""), true);
        assert_eq!(empty().starts_with("123"), false);
        assert_eq!(text("123").starts_with(""), true);
        assert_eq!(at(1).starts_with(""), true);
        assert_eq!(at(1).starts_with("1"), false);
        assert_eq!(msg!["123", at(1)].starts_with("123"), true);
        assert_eq!(msg!["123", at(1)].starts_with("122"), false);
    }
}
