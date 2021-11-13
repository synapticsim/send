#![feature(min_specialization)]
#![feature(negative_impls)]

use send::{receive, Actor, Framework};

#[derive(Actor)]
struct Root {
	data: Data,
	counter: u16,
	child: Child,
}

#[derive(Actor)]
struct Child {
	counter: u16,
	child: ChildChild,
}

#[derive(Actor)]
struct ChildChild {
	counter: u16,
}

struct Data {
	data: u16,
}

struct Increment(u16);

struct Decrement(u16);

receive! {
	Increment => Root = (&mut self, increment, _) {
		self.counter += increment.0;
	}
}

receive! {
	Increment => Child = (&mut self, message, _) {
		self.counter += message.0
	}

	Decrement => Child = (&mut self, message, _) {
		self.counter -= message.0
	}
}

receive! {
	Increment => ChildChild = (&mut self, message, context) {
		self.counter += message.0;

		context.broadcast(self, &mut Decrement(1));
	}
}

#[test]
fn test() {
	let mut framework = Framework::new(Root {
		data: Data { data: 1 },
		counter: 2,
		child: Child {
			counter: 2,
			child: ChildChild { counter: 2 },
		},
	});

	framework.send(&mut Increment(1));
	assert_eq!(framework.get().counter, 3);
	assert_eq!(framework.get().child.counter, 2);
	assert_eq!(framework.get().child.child.counter, 3);

	framework.send_sub(&mut Increment(1), |root| &mut root.child);
	assert_eq!(framework.get().counter, 3);
	assert_eq!(framework.get().child.counter, 2);
	assert_eq!(framework.get().child.child.counter, 4);

	framework.send_to(&mut Increment(1), |root| &mut root.child.child);
	assert_eq!(framework.get().counter, 3);
	assert_eq!(framework.get().child.counter, 1);
	assert_eq!(framework.get().child.child.counter, 5);

	framework.send_with(|root| &root.data, |data| Increment(data.data));
	assert_eq!(framework.get().counter, 4);
	assert_eq!(framework.get().child.counter, 1);
	assert_eq!(framework.get().child.child.counter, 6);
}
