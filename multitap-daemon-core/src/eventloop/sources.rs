use mio::{Events, Token};
use super::{Platform, EventSource, EventHandler};

pub struct EventSources<P: Platform> {
    poll: mio::Poll,
    slab: slab::Slab<P::Source>,
}

#[derive(Debug)]
pub enum Error {
    Mio(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self { Error::Mio(err) }
}

impl<P: Platform> std::ops::Index<Token> for EventSources<P> {
    type Output = P::Source;

    fn index(&self, token: Token) -> &Self::Output {
        &self.slab[token.0]
    }
}

impl<P: Platform> std::ops::IndexMut<Token> for EventSources<P> {

    fn index_mut(&mut self, token: Token) -> &mut Self::Output {
        &mut self.slab[token.0]
    }
}

impl<P: Platform> EventSources<P> {

    pub(super) fn new() -> Result<Self, Error> {
        let slab = slab::Slab::with_capacity(64);
        let poll = mio::Poll::new()?;

        Ok(Self { slab, poll })
    }

    pub(super) fn poll(&mut self, events: &mut Events) -> Result<(), Error> {
        self.poll.poll(events, None)?;
        Ok(())
    }

    pub fn register<H: EventHandler<Platform = P>>(&mut self, mut source: P::Source, handler: &mut H) -> Result<mio::Token, <P::Source as EventSource<P>>::Error> {
        let entry = self.slab.vacant_entry();
        let token = mio::Token(entry.key());
        source.register(self.poll.registry(), token, handler)?;
        entry.insert(source);
        Ok(token)
    }

    pub fn deregister<H: EventHandler<Platform = P>>(&mut self, token: mio::Token, handler: &mut H) -> Result<P::Source, <P::Source as EventSource<P>>::Error> {
        let mut source = self.slab.remove(token.0);
        source.deregister(self.poll.registry(), handler)?;
        Ok(source)
    }
}
