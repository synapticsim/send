# `send`

A library for efficient local message-passing 
(pretty much [actors](https://en.wikipedia.org/wiki/Actor_model) but without the concurrent part).

## Integrating

Add this to the `[dependencies]` of your `Cargo.toml`:
```toml
[dependencies]
send = { git = "https://github.com/Synaptic-Simulations/send", branch = "main" }
```

`send` requires nightly, and you have to add this to the top of your crate:
```rs
#![feature(min_specialization)]
```

## Using

`send` is a pretty small library because it has just one thing to do: safe, type-safe message passing.

### Actors, Messages, Events

Everything in `send` is an `Actor`. Actors can receive messages, send messages,
and contain sub-actors. Pretty simple.

To make something an actor, you simply:
```rs
use send::Actor;

#[derive(Actor)]
pub struct MyActor {}
```

What this does is register all sub-actors of this actor to be used for broadcasted messages.
 
Messages can be any type, without any special traits needing to be implemented. 

### Framework

The `Framework` is the 'root' of all Actors. 
It allows you to send external messages to all actors, a specific one, 
or a specific one and all its children.

Creating a `Framework` is done with `Framework::new()` and the root actor:
```rs
let framework: Framework<MyActor> = Framework::new(MyActor {});
```

### Receiving messages

To receive messages on an actor, simply implement the trait `send::Receiver` for your actor:
```rs
impl<T> Receiver<Message, T> for MyActor {
    fn receive(&mut self, message: &Message, context: Context<Self, T>) {
        // Do something with `message`
    }
}
```

Over here, `Message` is the type that you send, and `MyActor` is the type of the root actor.

This implements `Receiver` for *all* possible root actors.

Obviously this boilerplate is annoying, so enter macros!

Use the `send::receive` macro to implement `Receiver` for your actor:
```rs
use send::receive;

struct MyActor;

receive! {
    Message => MyActor = |&mut self, message, context| {
        // Your code here
    }
}
```

It even supports generics (but they're a bit clunky because macros can't really capture generics):
```rs
use send::receive;

struct MyMessage<const N: u8>;

recieve! {
    %(const N: u8) MyMessage<N> => MyActor = |&mut self, message, context| {
        // Your code here
    }
}
```

### Sending messages

In `receive` , `context` allows you to send messages to all actors,
a specific one, or a specific one and its sub-actors. 
These messages are sent and evaluated immediately.

Every method on `Context` requires you to pass `self` as the first parameter, for safety.
