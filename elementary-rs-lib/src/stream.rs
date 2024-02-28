use std::sync::mpsc;
use std::{
    pin::Pin,
    task::{Context, Poll},
};

use bevy::{app::App, prelude::*, tasks::futures_lite::Stream};

pub struct AppHtmlStream {
    app: App,
    page: Entity,
}

impl Stream for AppHtmlStream {
    type Item = String;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(Some(node))
    }
}
