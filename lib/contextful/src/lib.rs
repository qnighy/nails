use std::borrow::Cow;

pub trait Context: std::fmt::Debug + Clone {}

pub trait AsContext<U: Context>: Context {
    fn as_context(&self) -> Cow<'_, U>;
}

impl<T: Context> AsContext<T> for T {
    fn as_context(&self) -> Cow<'_, T> {
        Cow::Borrowed(self)
    }
}

#[cfg(test)]
mod tests {
    use self::external::*;
    use super::*;

    #[cfg(feature = "todo")]
    mod todo {
        // TODO: We want to write this kind of pattern:
        #[derive(Debug, Clone, Context)]
        pub struct CommonContext {
            #[as_context]
            time: TimeMocker,
            #[as_context]
            web: WebMocker,
            conn: PgConnection,
        }

        impl AsContext<MiddlewareContext> for CommonContext {
            fn as_context(&self) -> Cow<'_, MiddlewareContext> {
                Cow::from(MiddlewareContext {
                    time: self.time.clone(),
                    web: self.web.clone(),
                })
            }
        }

        #[derive(Debug, Clone, Context)]
        pub struct AppContext {
            #[as_context(TimeMocker, WebMocker)]
            common: CommonContext,
            server_config: ServerConfig,
        }

        #[derive(Debug, Clone, Context)]
        pub struct MiddlewareContext {
            #[as_context]
            time: TimeMocker,
            #[as_context]
            web: WebMocker,
        }
    }

    #[derive(Debug, Clone)]
    pub struct CommonContext {
        time: TimeMocker,
        web: WebMocker,
        conn: PgConnection,
    }

    impl Context for CommonContext {}

    impl AsContext<TimeMocker> for CommonContext {
        fn as_context(&self) -> Cow<'_, TimeMocker> {
            Cow::Borrowed(&self.time)
        }
    }

    impl AsContext<WebMocker> for CommonContext {
        fn as_context(&self) -> Cow<'_, WebMocker> {
            Cow::Borrowed(&self.web)
        }
    }

    impl AsContext<MiddlewareContext> for CommonContext {
        fn as_context(&self) -> Cow<'_, MiddlewareContext> {
            Cow::Owned(MiddlewareContext {
                time: self.time.clone(),
                web: self.web.clone(),
            })
        }
    }

    #[derive(Debug, Clone)]
    pub struct AppContext {
        common: CommonContext,
        server_config: ServerConfig,
    }

    impl Context for AppContext {}

    impl AsContext<CommonContext> for AppContext {
        fn as_context(&self) -> Cow<'_, CommonContext> {
            Cow::Borrowed(&self.common)
        }
    }

    impl AsContext<TimeMocker> for AppContext {
        fn as_context(&self) -> Cow<'_, TimeMocker> {
            self.common.as_context()
        }
    }

    impl AsContext<WebMocker> for AppContext {
        fn as_context(&self) -> Cow<'_, WebMocker> {
            self.common.as_context()
        }
    }

    #[derive(Debug, Clone)]
    pub struct MiddlewareContext {
        time: TimeMocker,
        web: WebMocker,
    }

    impl Context for MiddlewareContext {}

    impl AsContext<TimeMocker> for MiddlewareContext {
        fn as_context(&self) -> Cow<'_, TimeMocker> {
            Cow::Borrowed(&self.time)
        }
    }

    impl AsContext<WebMocker> for MiddlewareContext {
        fn as_context(&self) -> Cow<'_, WebMocker> {
            Cow::Borrowed(&self.web)
        }
    }

    mod external {
        #[derive(Debug, Clone)]
        pub struct TimeMocker(());

        #[derive(Debug, Clone)]
        pub struct WebMocker(());

        #[derive(Debug, Clone)]
        pub struct PgConnection(());

        #[derive(Debug, Clone)]
        pub struct ServerConfig(());

        impl super::Context for TimeMocker {}
        impl super::Context for WebMocker {}
        impl super::Context for PgConnection {}
        impl super::Context for ServerConfig {}
    }
}
