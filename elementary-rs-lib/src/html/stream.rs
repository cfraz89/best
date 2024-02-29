use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bevy::{prelude::*, tasks::futures_lite::Stream};
use either::Either;
use thiserror::Error;

use super::render::RenderOutput;

pub struct AppHtmlStream {
    app: App,
}
impl AppHtmlStream {
    pub fn new(app: App) -> Self {
        Self { app }
    }
}

#[derive(Error, Debug)]
pub enum Never {}

impl Stream for AppHtmlStream {
    type Item = Result<String, Never>;

    fn poll_next(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        self.app.update();
        let render_output = self.app.world.get_resource::<RenderOutput>().unwrap();
        match render_output.0.clone() {
            Either::Left(chunk) if chunk.len() == 0 => Poll::Pending, //TODO pass waker into waiting components to wake up
            Either::Left(chunk) => Poll::Ready(Some(Ok(chunk.to_string()))),
            Either::Right(rest) if rest.len() == 0 => Poll::Ready(None),
            Either::Right(rest) => Poll::Ready(Some(Ok(rest))),
        }
    }
}
