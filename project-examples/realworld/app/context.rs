use contextful::Context;

#[derive(Debug, Clone)]
pub struct AppCtx {
    _dummy: (),
}

impl Context for AppCtx {}

impl AppCtx {
    pub fn new() -> Self {
        Self { _dummy: () }
    }
}
