use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bevy::{prelude::*, tasks::futures_lite::Stream};
use either::Either;
use thiserror::Error;

use crate::r#async::AsyncContext;

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

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        // It will be reset to dropped during update
        self.app.insert_non_send_resource(AsyncContext::from(cx));
        self.app.update();
        let render_output = self.app.world.get_resource::<RenderOutput>().unwrap();
        match render_output.0.clone() {
            Either::Left(chunk) if chunk.len() == 0 => Poll::Pending, //TODO pass waker into waiting components to wake up
            Either::Left(chunk) => Poll::Ready(Some(Ok(format!(
                "{}{}\r\n",
                chunk.len(),
                chunk.to_string()
            )))),
            //Right means the stream is done
            Either::Right(rest) if rest.len() == 0 => Poll::Ready(None),
            Either::Right(rest) => Poll::Ready(Some(Ok(rest))),
        }
    }
}
