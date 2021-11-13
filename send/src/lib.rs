#![feature(auto_traits)]
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
	R: Actor + 'static,
{
	/// Create a [`Framework`] handling a root [`Actor`].
	pub fn new(root: R) -> Self { Self { root } }

	/// Send a message to every [`Actor`] in the [`Framework`].
	pub fn send<M>(&mut self, message: &mut M) {
		let mut visitor = MessageVisitor {
			message,
			root: &mut self.root as *mut _,
		};
		self.root.accept(&mut visitor);
	}

	/// Send a message to only a specific [`Actor`].
	///
	/// `getter`: A function that takes in the root and outputs the [`Actor`] to send the message to.
	pub fn send_to<M, F, A>(&mut self, message: &mut M, getter: F)
	where
		F: FnOnce(&mut R) -> &mut A,
	{
		let mut visitor = MessageVisitor {
			message,
			root: &mut self.root as *mut _,
		};
		visitor.visit(getter(&mut self.root));
	}

	/// Send a message to a specific [`Actor`] and its sub-[`Actor`]s.
	///
	/// `getter`: A function that takes in the root and outputs the [`Actor`] to send the message to.
	pub fn send_sub<M, F, A>(&mut self, message: &mut M, getter: F)
	where
		F: FnOnce(&mut R) -> &mut A,
	{
		let mut visitor = MessageVisitor {
			message,
			root: &mut self.root as *mut _,
		};
		getter(&mut self.root).accept(&mut visitor);
	}

	/// Send a message that contains references to fields or sub-fields.
	/// This sends the message to every [`Actor`] in the [`Framework`].
	///
	/// `selector`: A function that selects the fields to contain in the message.  
	/// `creator`: A function that generates the message to send.
	pub fn send_with<'a, S, F, C, M>(&'a mut self, selector: S, creator: C)
	where
		S: FnOnce(&'a mut R) -> F,
		F: 'a + NotActor,
		C: FnOnce(F) -> M,
	{
		// SAFETY: We are verifying that the selected fields do not respond to any messages with the `NotActor` trait
		// bound. Implementors of `Actor` must ensure that they un-implement `NotActor` if they or sub-`Actor`s respond
		// to any messages. It is an `unsafe` trait, so it's not our problem if they mess up.
		let fields = selector(unsafe { &mut *(&mut self.root as *mut _) });
		self.send(&mut creator(fields));
	}

	/// Send a message that contains references to fields or sub-fields.
	/// This sends a message to only a specific [`Actor`].
	///
	/// `selector`: A function that selects the fields to contain in the message.  
	/// `creator`: A function that generates the message to send.  
	/// `getter`: A function that takes in the root and outputs the [`Actor`] to send the message to.
	pub fn send_to_with<'a, S, F, C, M, G, A>(&'a mut self, selector: S, creator: C, getter: G)
	where
		S: FnOnce(&'a mut R) -> F,
		F: 'a + NotActor,
		C: FnOnce(F) -> M,
		G: FnOnce(&mut R) -> &mut A,
	{
		// SAFETY: Above.
		let fields = selector(unsafe { &mut *(&mut self.root as *mut _) });
		self.send_to(&mut creator(fields), getter);
	}

	/// Send a message that contains references to fields or sub-fields.
	/// This sends a message to a specific [`Actor`] and its sub-[`Actor`]s.
	///
	/// `selector`: A function that selects the fields to contain in the message.  
	/// `creator`: A function that generates the message to send.  
	/// `getter`: A function that takes in the root and outputs the [`Actor`] to send the message to.
	pub fn send_sub_with<'a, S, F, C, M, G, A>(&'a mut self, selector: S, creator: C, getter: G)
	where
		S: FnOnce(&'a mut R) -> F,
		F: 'a + NotActor,
		C: FnOnce(F) -> M,
		G: FnOnce(&mut R) -> &mut A,
	{
		// SAFETY: Above.
		let fields = selector(unsafe { &mut *(&mut self.root as *mut _) });
		self.send_sub(&mut creator(fields), getter);
	}

	/// Get a reference to the root [`Actor`].
	pub fn get(&self) -> &R { &self.root }

	/// Get a mutable reference to the root [`Actor`].
	/// This shouldn't be used very often: prefer sending events instead.
	pub fn get_mut(&mut self) -> &mut R { &mut self.root }
}

struct MessageVisitor<'a, M, R> {
	message: &'a mut M,
	root: *mut R,
}

impl<M, R> ActorVisitor<M, R> for MessageVisitor<'_, M, R> {
	#[inline(always)]
	fn visit<A>(&mut self, actor: &mut A)
	where
		A: Actor + Receiver<M, R>,
	{
		let context = Context::new(self.root);
		actor.receive(self.message, context);
	}
}

/// A macro for easily implementing [`Receiver`] for your types.
///
/// ## Examples:
/// A type without generics:
/// ```
/// # #![feature(min_specialization)]
/// # use send::receive;
///
/// struct MyActor;
/// struct MyMessage;
///
/// receive! {
/// 	MyMessage => MyActor = (&mut self, _message, _context) {
/// 		// Some code here
/// 	}
/// }
/// ```
///
/// With generics:
/// ```
/// # #![feature(min_specialization)]
/// # use send::receive;
///
/// struct MyActor;
/// struct MyMessage<const Idx: u8>;
///
/// receive! {
/// 	%(const Idx: u8) MyMessage<Idx> => MyActor = (&mut self, _message, _context) {
/// 		// Some code here
/// 	}
/// }
/// ```
/// or just:
/// ```
/// # #![feature(min_specialization)]
/// # use send::receive;
///
/// struct MyActor;
/// struct MyMessage<const Idx: u8>;
///
/// receive! {
/// 	MyMessage<1> => MyActor = (&mut self, _message, _context) {
/// 		// Some code here
/// 	}
/// }
/// ```
///
/// Note the funky `%(...)` syntax. This is due to declarative macro limitations.
#[macro_export]
macro_rules! receive {
	($(%$generics:tt)? $message_ty:ty => $on:ty = (&mut $self:ident, $message:pat, $context:pat) $code:block $($rest:tt)*) => {
		$crate::receive! { $message_ty, $on, $self, $message, $context, $code, $($generics)? }

		$crate::receive! { $($rest)* }
	};

	($message_ty:ty, $on:ty, $self:ident, $message:pat, $context:pat, $code:block, $( ( $($generics:tt)* ) )?) => {
        impl<_RootTy, $($($generics)*)?> $crate::Receiver<$message_ty, _RootTy> for $on {
            fn receive(&mut $self, $message: &mut $message_ty, $context: $crate::Context<$on, _RootTy>) $code
        }
    };

	() => {};
}
