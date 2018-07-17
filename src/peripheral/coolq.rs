use std::path::{Path, PathBuf};
use failure::{err_msg, Error};
use composer::Composer;
use msg::{Msg, ExtBuilder, MsgBuilder};

fn extend_esc(string: &str, out: &mut String) {
    for c in string.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '[' => out.push_str("&#91;"),
            ']' => out.push_str("&#93;"),
            c => out.push(c),
        }
    }
}
fn extend_esc_cq(string: &str, out: &mut String) {
    for c in string.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '[' => out.push_str("&#91;"),
            ']' => out.push_str("&#93;"),
            ',' => out.push_str("&#44;"),
            c => out.push(c),
        }
    }
}
fn inverse(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&#91;", "[")
        .replace("&#93;", "]")
}
fn inverse_cq(text: &str) -> String {
    text.replace("&amp;", "&")
        .replace("&#91;", "[")
        .replace("&#93;", "]")
        .replace("&#44;", ",")
}

pub struct CoolQComposer {
    data_dir: PathBuf,
}
impl CoolQComposer {
    pub fn new<T>(data_dir: &T) -> CoolQComposer
            where T: ?Sized + AsRef<Path> {
        let mut data_dir = data_dir.as_ref().to_owned();
        data_dir.push("data");
        CoolQComposer {
            data_dir: data_dir,
        }
    }
    fn compose_impl(&self, msg: &Msg, out: &mut String) -> Result<(), Error> {
        match msg {
            Msg::Text(ref content) => {
                extend_esc(content, out);
            },
            Msg::Ext { name: name, params: params } => {
                out.push_str("[CQ:");
                extend_esc_cq(&name, out);
                for param in params.iter() {
                    out.push(',');
                    extend_esc_cq(param.0, out);
                    out.push('=');
                    // Translate path.
                    if param.0 == "file" {
                        let mut prefix = self.data_dir.clone();
                        prefix.push(&name);
                        extend_esc_cq(
                            &Path::new(param.1)
                                .strip_prefix(&prefix)?
                                .to_string_lossy(),
                            out
                        );
                    } else {
                        extend_esc_cq(param.1, out);
                    }
                }
                out.push(']');
            },
            Msg::Compound(segs) => {
                for seg in segs {
                    self.compose_impl(seg, out)?;
                }
            }
        }
        Ok(())
    }
    fn parse_cq(&self, string: &str) -> Result<Msg, Error> {
        let mut iter = string.split(',');
        let name = inverse_cq(iter.next()
            .ok_or_else(|| err_msg("unable to parse cq code"))?);
        let mut out_params = ExtBuilder::new(&name);
        for param in iter {
            let mut param = param.splitn(2, '=');
            let key = inverse_cq(param.next().unwrap().trim());
            let mut value = inverse_cq(param.next()
                .ok_or_else(|| err_msg("missing parameter value"))?.trim());
            // Translate path.
            if key == "file" {
                let mut path = self.data_dir.clone();
                path.push(&name);
                path.push(value);
                out_params.add_param(&key, &path.to_string_lossy());
            } else {
                out_params.add_param(&key, &value);
            }
        }
        Ok(out_params.build())
    }
}
impl Composer for CoolQComposer {
    fn name(&self) -> &'static str {
        "composer.coolq"
    }
    fn compose(&self, msg: &Msg) -> Result<String, Error> {
        let mut out = String::new();
        self.compose_impl(msg, &mut out)?;
        Ok(out)
    }
    fn decompose(&self, raw: &str) -> Result<Msg, Error> {
        use msg::text;
        let mut beg = 0;
        let mut rv = MsgBuilder::new();
        while (beg < raw.len()) {
            if let Some(from) = raw[beg..].find("[CQ:") {
                if (from > 0) {
                    rv.add_msg(Msg::Text(inverse(&raw[beg..(beg + from)])));
                }
                beg += from + 4; // Skip `[CQ:`.
                let to = raw[beg..].find(']')
                    .ok_or_else(|| err_msg("unclosed cq code"))?;
                let cq = self.parse_cq(&raw[beg..(beg + to)])?;
                rv.add_msg(cq);
                beg += to + 1; // Skip `]`.
            } else {
                // Couldn't find a next CQ code.
                break;
            }
        }
        // Add the remaining segment.
        if (beg < raw.len()) {
            rv.add_msg(Msg::Text(inverse(&raw[beg..])));
        }
        Ok(rv.build())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use msg::*;
    fn make_composer() -> CoolQComposer {
        CoolQComposer::new("C:/")
    }
    fn test_simple() {
        let composer = make_composer();
        let ext = ExtBuilder::new("x").with_param("y", "123")
                                      .build();
        let raw = "123[CQ:x,y=123]";
        let msg = msg!["123", ext];
        assert_eq!(raw, composer.compose(&msg).unwrap());
        assert_eq!(msg, composer.decompose(&raw).unwrap());
    }
    #[test]
    fn test_escape() {
        let composer = make_composer();
        let ext = ExtBuilder::new("x").with_param("y", "&[],")
                                      .build();
        let raw = ",&amp;&#91;&#93;[CQ:x,y=&amp;&#91;&#93;&#44;]";
        let msg = msg![",&[]", ext];
        assert_eq!(raw, composer.compose(&msg).unwrap());
        assert_eq!(msg, composer.decompose(&raw).unwrap());
    }
    #[test]
    fn test_compose_path_translation() {
        let composer = make_composer();
        let path: PathBuf = ["C:/", "data", "image", "1.jpg"].iter().collect();
        let raw = "[CQ:image,file=1.jpg]";
        let msg = image(&path);
        assert_eq!(raw, composer.compose(&msg).unwrap());
        assert_eq!(msg, composer.decompose(&raw).unwrap());
    }
}
