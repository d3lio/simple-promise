use std::sync::mpsc::{channel, Sender, Receiver};

#[derive(Debug, Eq, PartialEq)]
pub enum PromiseResult<T: Send> {
    Pending,
    Resolved(T),
    Rejected(T),
}

pub struct Promise<T: Send> {
    #[allow(dead_code)]
    receiver: Receiver<PromiseResult<T>>,
}

impl<T: Send> Promise<T> {
    pub fn new<F>(executor: F) -> Self
    where
        F: FnOnce(Sender<PromiseResult<T>>)
    {
        let (sender, receiver) = channel();

        executor(sender);

        Self {
            receiver,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn executes() {
        let mut called = false;
        let _ = Promise::<u32>::new(|_| called = true);
        assert!(called);
    }

    #[test]
    fn resolves() {
        let p = Promise::<u32>::new(|sender| {
            let _ = sender.send(PromiseResult::Resolved(1));
        });

        assert_eq!(p.receiver.recv(), Ok(PromiseResult::Resolved(1)));
    }

    #[test]
    fn rejects() {
        let p = Promise::<u32>::new(|sender| {
            let _ = sender.send(PromiseResult::Rejected(1));
        });


        assert_eq!(p.receiver.recv(), Ok(PromiseResult::Rejected(1)));
    }

    #[test]
    fn thread() {
        let mut handle = None;
        let p = Promise::<u32>::new(|sender| {
            handle = Some(std::thread::spawn(move || {
                let _ = sender.send(PromiseResult::Rejected(1));
            }));
        });

        assert!(handle.is_some());

        handle.unwrap().join().unwrap();

        assert_eq!(p.receiver.recv(), Ok(PromiseResult::Rejected(1)));
    }
}
