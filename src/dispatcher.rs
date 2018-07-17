//! Dispatcher for routing of all message backends.
use std::cell::Cell;
use std::os::raw::c_char;
use failure::Error;
use {Backend, Composer, Msg, MsgIn};

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

    pub fn composer(&self) -> &Composer {
        &*self.composer
    }

    pub fn use_composer<C>(&mut self, composer: C) -> &mut Dispatcher
            where C: 'static + Composer {
        self.composer = Box::new(composer);
        self
    }
    pub fn use_backend<B>(&mut self, backend: B, priority: i32)
            -> &mut Dispatcher where B: 'static + Backend {
        self.backends.push(Box::new(backend));
        self
    }
    pub fn dispatch(&self, msg_in: MsgIn) -> Option<Msg> {
        let is_priv = msg_in.is_priv();
        for backend in self.backends.iter() {
            if backend.preview(&msg_in) {
                match backend.process(&msg_in) {
                    Ok(msg) => return Some(msg),
                    _ => continue,
                }
            }
        }
        None
    }
}

struct DefaultComposer();
impl Composer for DefaultComposer {
    fn name(&self) -> &'static str {
        "composer.default"
    }
    fn compose(&self, msg: &Msg) -> Result<String, Error> {
        let rv = match msg {
            Msg::Text(ref content) => content.to_owned(),
            Msg::Compound(ref segs) => {
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
    fn decompose(&self, raw: &str) -> Result<Msg, Error> {
        Ok(::msg::text(raw))
    }
}
