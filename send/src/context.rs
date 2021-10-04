use std::marker::PhantomData;

use crate::{Actor, ActorVisitor, EventReceiver, Framework, Receiver};

/// A context that give you access to the [`Framework`] from inside an [`Actor`].
pub struct Context<S, R> {
	framework: *mut Framework<R>,
	phantom: PhantomData<S>,
}

impl<S, R> Context<S, R> {
	pub fn new(framework: *mut Framework<R>) -> Self {
		Self {
			framework,
			phantom: PhantomData,
		}
	}
}

impl<S, R> Context<S, R>
where
	R: Actor,
{
	/// Broadcast a message to all the [`Actor`]s in the [`Framework`].
	pub fn broadcast<T>(&self, _from: &mut S, message: &T) {
		// This is safe because `from` was the only `Actor` that had a mutable reference taken to it.
		// Since we now have a mutable reference to `from`, we can mutate the `Framework` freely.
		let framework = unsafe { &mut *self.framework };
		let mut visitor = MessageVisitor { message, framework };
		framework.root.accept(&mut visitor);
	}

	/// Send a message to only a specific [`Actor`].
	///
	/// `getter`: A function that takes in the root and outputs the [`Actor`] to send the event to.
	pub fn send<T, F, A>(&self, _from: &mut S, message: &T, getter: F)
	where
		A: Actor + Receiver<T, R> + EventReceiver<T, R>,
		F: FnOnce(&mut R) -> &mut A,
	{
		// Above ^^
		let framework = unsafe { &mut *self.framework };
		let mut visitor = MessageVisitor { message, framework };
		visitor.visit(getter(&mut framework.root));
	}

	/// Send a message to a specific [`Actor`] and its sub-[`Actor`]s.
	///
	/// `getter`: A function that takes in the root and outputs the [`Actor`] to send the event to.
	pub fn send_sub<T, F, A>(&self, _from: &mut S, message: &T, getter: F)
	where
		A: Actor + Receiver<T, R> + EventReceiver<T, R>,
		F: FnOnce(&mut R) -> &mut A,
	{
		// Above ^^
		let framework = unsafe { &mut *self.framework };
		let mut visitor = MessageVisitor { message, framework };
		getter(&mut framework.root).accept(&mut visitor);
	}
}

struct MessageVisitor<'a, M, R> {
	message: &'a M,
	framework: *mut Framework<R>,
}

impl<M, R> ActorVisitor<M, R> for MessageVisitor<'_, M, R> {
	fn visit<A>(&mut self, actor: &mut A)
	where
		A: Actor + Receiver<M, R> + EventReceiver<M, R>,
	{
		let context = Context::new(self.framework);
		actor.receive(&self.message, context);
	}
}
