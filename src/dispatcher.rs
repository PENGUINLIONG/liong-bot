//! Dispatcher for routing of all message backends.
use std::cell::Cell;
use std::os::raw::c_char;
use failure::Error;
use {Backend, Composer, Msg};

pub struct Dispatcher {
    composer: Box<Composer>,
    enabled: Cell<bool>,
    backends: Vec<Box<Backend>>,
}
impl Dispatcher {
    pub fn new() -> Dispatcher {
        Dispatcher {
            composer: Box::new(DefaultComposer()),
            enabled: Cell::new(false),
            backends: Vec::new(),
        }
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled.get()
    }
    pub fn is_disabled(&self) -> bool {
        !self.enabled.get()
    }
    pub fn enable(&self) {
        self.enabled.set(true);
    }
    pub fn disable(&self) {
        self.enabled.set(false);
    }

    pub fn with_composer<C>(&mut self, composer: C) -> &mut Dispatcher
            where C: 'static + Composer {
        self.composer = Box::new(composer);
        self
    }
    pub fn with_backend<B>(&mut self, backend: B, priority: i32)
            -> &mut Dispatcher where B: 'static + Backend {
        self.backends.push(Box::new(backend));
        self
    }
    pub fn forward_priv(&self, qq: i64, raw: &str) {
        // TODO: Implement.
    }
}

struct DefaultComposer();
impl Composer for DefaultComposer {
    fn name(&self) -> &'static str {
        "composer.default"
    }
    fn compose(&self, msg: Msg) -> Result<String, Error> {
        let rv = match msg {
            Msg::Text(content) => content,
            Msg::Compound(segs) => {
                let mut rv = String::new();
                for seg in segs {
                    rv.push_str(&self.compose(seg)?);
                }
                rv
            },
            _ => String::new(),
        };
        Ok(rv)
    }
    fn decompose(&self, raw: String) -> Result<Msg, Error> {
        Ok(Msg::Text(raw.to_owned()))
    }
}
