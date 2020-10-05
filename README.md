## Example

```rust
use pakr_fsm::*;
use std::error;

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum MyEv {
    E1,
    E2,
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum MyState {
    S1,
    S2,
}

impl Default for MyState {
    fn default() -> Self { Self::S1 }
}

struct MyFSM;

impl FSM for MyFSM {
    type Event = MyEv;
    type Response = &'static str;
    type State = MyState;

    fn new() -> Self { Self {} }

    fn trasnsit(
        &mut self,
        old_state: &Self::State,
        ev: &Self::Event,
    ) -> (Option<Self::State>, Option<Self::Response>) {
        match (old_state, ev) {
            (MyState::S1, MyEv::E1) => (None, Some("Quitting")),
            (MyState::S1, MyEv::E2) => (Some(MyState::S2), None),
            (MyState::S2, MyEv::E1) => (Some(MyState::S1), Some("S2@E1->S1")),
            (MyState::S2, MyEv::E2) => (Some(MyState::S2), Some("S2@E2->S2")),
        }
    }

    fn respond(
        &mut self,
        old_state: &<Self as FSM>::State,
        new_state: &Option<<Self as FSM>::State>,
        resp: &Self::Response,
    ) {
        println!("{:?} -> ({:?},{:?}", old_state, new_state, resp);
    }
}

fn main() -> Result<(), Box<dyn error::Error>> {
    let fsm = Reactor::<MyFSM>::new();
    fsm.send(MyEv::E2)?;
    fsm.send(MyEv::E2)?;
    fsm.send(MyEv::E1)?;
    fsm.send(MyEv::E1)?;
    let ans = fsm.join().unwrap();
    println!("Final state result: {:?}",ans);
    Ok(())
}
```
