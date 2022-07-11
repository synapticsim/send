use std::marker::PhantomData;

use crate::{Actor, ActorVisitor, MessageVisitor, NotActor, Receiver};

/// A context that give you access to the [`Framework`](super::Framework) from inside an [`Actor`].
pub struct Context<S, R> {
	root: *mut R,
	phantom: PhantomData<*const S>,
}

impl<S, R> Context<S, R> {
	pub fn new(root: *mut R) -> Self {
		Self {
			root,
			phantom: PhantomData,
		}
	}
}

impl<S, R> Context<S, R>
where
	S: 'static,
	R: Actor,
{
	/// Broadcast a message to all the [`Actor`]s in the [`Framework`](super::Framework).
	#[inline(always)]
	pub fn broadcast<T>(&self, _from: &mut S, message: &mut T) {
		let mut visitor = MessageVisitor {
			message,
			root: self.root,
		};
		// SAFETY:
		// This is safe because `from` was the only `Actor` that had a mutable reference taken to it.
		// Since we now have a mutable reference to `from`, we can mutate the `Framework`.
		unsafe {
			(*self.root).accept(&mut visitor);
		}
	}

	/// Send a message to only a specific [`Actor`].
	///
	/// `getter`: A function that takes in `Self` and outputs the [`Actor`] to send the event to.
	#[inline(always)]
	pub fn send<T, F, A>(&self, from: &mut S, message: &mut T, getter: F)
	where
		A: Actor + Receiver<T, R>,
		F: FnOnce(&mut S) -> &mut A,
	{
		let mut visitor = MessageVisitor {
			message,
			root: self.root,
		};
		visitor.visit(getter(from))
	}

	/// Send a message to a specific [`Actor`] and its sub-[`Actor`]s.
	///
	/// `getter`: A function that takes in `Self` and outputs the [`Actor`] to send the event to.
	#[inline(always)]
	pub fn send_sub<T, F, A>(&self, from: &mut S, message: &mut T, getter: F)
	where
		A: Actor + Receiver<T, R>,
		F: FnOnce(&mut S) -> &mut A,
	{
		let mut visitor = MessageVisitor {
			message,
			root: self.root,
		};

		getter(from).accept(&mut visitor);
	}

	/// Send a message that contains references to fields or sub-fields.
	/// This sends the message to every [`Actor`] in the [`Framework`](super::Framework).
	///
	/// `selector`: A function that selects the fields to contain in the message.  
	/// `creator`: A function that generates the message to send.
	pub fn broadcast_with<'a, Sel, F, C, M>(&self, from: &'a mut S, selector: Sel, creator: C)
	where
		Sel: FnOnce(&'a mut S) -> F,
		F: 'a + NotActor,
		C: FnOnce(F) -> M,
	{
		let fields = selector(unsafe { &mut *(from as *mut S) });
		self.broadcast(from, &mut creator(fields));
	}

	/// Send a message that contains references to fields or sub-fields.
	/// This sends a message to only a specific [`Actor`].
	///
	/// `selector`: A function that selects the fields to contain in the message.  
	/// `creator`: A function that generates the message to send.  
	/// `getter`: A function that takes in `Self` and outputs the [`Actor`] to send the message to.
	pub fn send_with<'a, Sel, F, C, M, G, A>(&self, from: &'a mut S, selector: Sel, creator: C, getter: G)
	where
		Sel: FnOnce(&'a mut S) -> F,
		F: 'a + NotActor,
		C: FnOnce(F) -> M,
		G: FnOnce(&mut S) -> &mut A,
	{
		let fields = selector(unsafe { &mut *(from as *mut S) });
		self.send(from, &mut creator(fields), getter);
	}

	/// Send a message that contains references to fields or sub-fields.
	/// This sends a message to a specific [`Actor`] and its sub-[`Actor`]s.
	///
	/// `selector`: A function that selects the fields to contain in the message.  
	/// `creator`: A function that generates the message to send.  
	/// `getter`: A function that takes in`Self` and outputs the [`Actor`] to send the message to.
	pub fn send_sub_with<'a, Sel, F, C, M, G, A>(&self, from: &'a mut S, selector: Sel, creator: C, getter: G)
	where
		Sel: FnOnce(&'a mut S) -> F,
		F: 'a + NotActor,
		C: FnOnce(F) -> M,
		G: FnOnce(&mut S) -> &mut A,
	{
		let fields = selector(unsafe { &mut *(from as *mut S) });
		self.send_sub(from, &mut creator(fields), getter);
	}
}
