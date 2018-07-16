use std::path::PathBuf;
use failure::{err_msg, Error};
use composer::Composer;
use msg::{Msg, ExtBuilder, CompoundBuilder};

pub struct CoolQComposer {
    data_dir: PathBuf,
}
impl Composer for CoolQComposer {
    fn name(&self) -> &'static str {
        "composer.coolq"
    }
    fn compose(&self, msg: Msg) -> Result<String, Error> {
        fn compose_impl(msg: Msg, out: &mut String) {
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
            match msg {
                Msg::Text(content) => {
                    extend_esc(&content, out);
                },
                Msg::Ext { name: name, params: params } => {
                    out.push_str("[CQ:");
                    extend_esc_cq(&name, out);
                    for param in params.iter() {
                        out.push(',');
                        extend_esc_cq(param.0, out);
                        out.push('=');
                        extend_esc_cq(param.1, out);
                    }
                    out.push(']');
                },
                Msg::Compound(segs) => {
                    for seg in segs {
                        compose_impl(seg, out);
                    }
                }
            }
        }
        let mut out = String::new();
        compose_impl(msg, &mut out);
        Ok(out)
    }
    fn decompose(&self, raw: String) -> Result<Msg, Error> {
        use msg::text;
        fn parse_cq(string: &str) -> Result<Msg, Error> {
            let mut iter = string.split(',');
            let name = inverse_cq(iter.next()
                .ok_or_else(|| err_msg("unable to parse cq code"))?);
            let mut out_params = ExtBuilder::new(&name);
            for param in iter {
                let mut param = param.splitn(2, '=');
                let key = inverse_cq(param.next().unwrap());
                let value = inverse_cq(param.next()
                    .ok_or_else(|| err_msg("missing parameter value"))?);
                out_params.add_param(&key, &value);
            }
            Ok(out_params.build())
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

        let mut beg = 0;
        let mut rv = CompoundBuilder::new();
        while (beg < raw.len()) {
            if let Some(from) = raw[beg..].find("[CQ:") {
                if (from > 0) {
                    rv.add_msg(Msg::Text(inverse(&raw[beg..(beg + from)])));
                }
                beg += from + 4; // Skip `[CQ:`.
                let to = beg + raw[beg..].find(']')
                    .ok_or_else(|| err_msg("unclosed cq code"))?;
                let cq = parse_cq(&raw[beg..(beg + to)])?;
                rv.add_msg(cq);
                beg = to + 1; // Skip `]`.
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
