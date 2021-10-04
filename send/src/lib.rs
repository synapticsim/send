#![feature(min_specialization)]

mod actor;
mod context;

pub use actor::*;
pub use context::*;

/// The root of everything.
///
/// It handles a root [`Actor`] and all its sub-[`Actor`]s,
/// and facilitates message-passing between them, as well as external events.
pub struct Framework<R> {
	root: R,
}

impl<R> Framework<R>
where
	R: Actor,
{
	/// Create a [`Framework`] handling a root [`Actor`].
	pub fn new(root: R) -> Self { Self { root } }

	/// Send an event to every [`Actor`] in the [`Framework`].
	pub fn send_event<E>(&mut self, event: &mut E) {
		let mut visitor = EventVisitor { event, framework: self };
		self.root.accept(&mut visitor);
	}

	/// Send an event to only a specific [`Actor`].
	///
	/// `getter`: A function that takes in the root and outputs the [`Actor`] to send the event to.
	pub fn send_event_to<E, F, A>(&mut self, event: &mut E, getter: F)
	where
		A: Actor + Receiver<E, R> + EventReceiver<E, R>,
		F: FnOnce(&mut R) -> &mut A,
	{
		let mut visitor = EventVisitor { event, framework: self };
		visitor.visit(getter(&mut self.root));
	}

	/// Send an event to a specific [`Actor`] and its sub-[`Actor`]s.
	///
	/// `getter`: A function that takes in the root and outputs the [`Actor`] to send the event to.
	pub fn send_event_sub<E, F, A>(&mut self, event: &mut E, getter: F)
	where
		A: Actor + Receiver<E, R> + EventReceiver<E, R>,
		F: FnOnce(&mut R) -> &mut A,
	{
		let mut visitor = EventVisitor { event, framework: self };
		getter(&mut self.root).accept(&mut visitor);
	}

	/// Get a reference to the root [`Actor`].
	pub fn get(&self) -> &R { &self.root }
}

struct EventVisitor<'a, E, R> {
	event: &'a mut E,
	framework: *mut Framework<R>,
}

impl<E, R> ActorVisitor<E, R> for EventVisitor<'_, E, R> {
	fn visit<A>(&mut self, actor: &mut A)
	where
		A: Actor + Receiver<E, R> + EventReceiver<E, R>,
	{
		let context = Context::new(self.framework);
		actor.receive_event(self.event, context);
	}
}
