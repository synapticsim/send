#![feature(min_specialization)]

mod external_messages {
	use send::{Actor, Context, Framework, Receiver};

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

	impl Receiver<Increment, Root> for Root {
		fn receive(&mut self, message: &mut Increment, _: Context<Self, Root>) { self.counter += message.0; }
	}

	impl Receiver<Increment, Root> for ChildChild {
		fn receive(&mut self, message: &mut Increment, _: Context<Self, Root>) { self.counter += message.0; }
	}

	impl Receiver<Increment, Root> for Child {
		fn receive(&mut self, message: &mut Increment, _: Context<Self, Root>) { self.counter += message.0; }
	}

	#[test]
	fn test() {
		let mut framework = Framework::new(Root {
			data: Data { data: 1 },
			counter: 0,
			child: Child {
				counter: 0,
				child: ChildChild { counter: 0 },
			},
		});

		framework.send(&mut Increment(1));
		assert_eq!(framework.get().counter, 1);
		assert_eq!(framework.get().child.counter, 1);
		assert_eq!(framework.get().child.child.counter, 1);

		framework.send_sub(&mut Increment(1), |root| &mut root.child);
		assert_eq!(framework.get().counter, 1);
		assert_eq!(framework.get().child.counter, 2);
		assert_eq!(framework.get().child.child.counter, 2);

		framework.send_to(&mut Increment(1), |root| &mut root.child.child);
		assert_eq!(framework.get().counter, 1);
		assert_eq!(framework.get().child.counter, 2);
		assert_eq!(framework.get().child.child.counter, 3);

		framework.send_with(|root| &root.data, |data| Increment(data.data));
		assert_eq!(framework.get().counter, 2);
		assert_eq!(framework.get().child.counter, 3);
		assert_eq!(framework.get().child.child.counter, 4);
	}
}
