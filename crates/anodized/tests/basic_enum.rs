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

#[spec]
impl Job {
    #[spec(
        requires: matches!(self.state, State::Idle),
        maintains: matches!(self.state, State::Idle | State::Running | State::Finished),
        ensures: matches!(self.state, State::Running),
    )]
    fn start(&mut self) {
        self.state = State::Running;
    }
}

#[test]
fn job_start_success() {
    let mut job = Job { state: State::Idle };
    job.start();
}

#[cfg(all(anodized_print, anodized_panic))]
#[test]
#[should_panic(expected = "precondition failed")]
fn job_start_panics_if_not_idle() {
    let mut job = Job {
        state: State::Running,
    };
    job.start(); // This violates the precondition.
}
