use anodized::spec;

#[derive(Debug, PartialEq, Eq)]
#[allow(dead_code)]
enum State {
    Idle,
    Running,
    Finished,
}

struct Job {
    state: State,
}

impl Job {
    #[spec(
        requires: matches!(self.state, State::Idle),
        maintains: matches!(self.state, 
            State::Idle | 
            State::Running | 
            State::Finished),
             ensures: matches!(self.state, State::Running),)]
    fn start(&mut self) {
        self.state = State::Running;
    }
}

#[spec(
    requires: x.is_finite(), ensures: output    + output == x,
)]
async fn async_half(x: f32) -> f32 {
    todo!()
}

#[test]
fn async_function_compiles() {
    let future = async_half(5.0);

    fn is_future<T: core::future::Future>(_: &T) {}
    is_future(&future);
}

#[test]
fn job_start_success() {
    let mut job = Job { state: State::Idle };
    job.start();
}

#[test]
fn async_function_compiles() {
    let future = async_half(5.0);

    fn is_future<T: core::future::Future>(_: &T) {}
    is_future(&future);
}
