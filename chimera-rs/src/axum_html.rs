use crate::html::{plugin::RenderHtmlPlugin, stream::AppHtmlStream};
use axum_core::{
    body::Body,
    response::{IntoResponse, Response},
};
use bevy::{ecs::schedule::ScheduleLabel, prelude::*};

pub struct AxumHtmlApp {
    pub app: App,
}

impl AxumHtmlApp {
    pub fn new<S>(init: impl IntoSystemConfigs<S>) -> Self {
        let mut app = App::new();
        app.add_plugins(RenderHtmlPlugin);
        app.add_systems(Startup, init);
        Self { app }
    }

    pub fn add_systems<S>(
        &mut self,
        schedule: impl ScheduleLabel,
        systems: impl IntoSystemConfigs<S>,
    ) -> &mut Self {
        self.app.add_systems(schedule, systems);
        self
    }
}

impl IntoResponse for AxumHtmlApp {
    fn into_response(self) -> Response {
        Body::from_stream(AppHtmlStream::new(self.app)).into_response()
    }
}
