use std::cell::RefCell;

type Resolve<T> = dyn FnOnce(T);
type Reject<E> = dyn FnOnce(E);
type Executor<T, E> = dyn Fn(Box<Resolve<T>>, Box<Reject<E>>);

struct InternalPromise<T, E> {
    result: Option<Result<T, E>>,
}

pub struct Promise<T, E> {
    internal: RefCell<InternalPromise<T, E>>,
}


impl<T: 'static, E: 'static> InternalPromise<T, E> {
    fn new() -> Self {
        Self {
            result: None,
        }
    }

    fn is_done(&self) -> bool {
        self.result.is_some()
    }

    fn resolve(&mut self, value: Option<T>) {
        self.result = value.map(Ok);
    }

    fn reject(&mut self, reason: Option<E>) {
        self.result = reason.map(Err);
    }
}

impl<T: 'static, E: 'static> Promise<T, E> {
    pub fn new<F>(executor: F) -> Self
    where
        F: FnOnce(Box<Resolve<T>>, Box<Reject<E>>)
    {
        let internal = RefCell::new(InternalPromise::new());

        executor(
            Box::new(|value| internal.borrow_mut().resolve(Some(value))),
            Box::new(|reason| internal.borrow_mut().reject(Some(reason))),
        );

        Self {
            internal,
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn executes() {
        let mut called = false;
        let _ = Promise::<u32, u32>::new(|_, _| called = true);
        assert!(called);
    }

    #[test]
    fn resolves() {
        let p = Promise::<u32, u32>::new(|resolve, _| {
            resolve(1);
        });

        assert!(p.internal.result.is_some());
        assert!(p.internal.result.unwrap().is_ok());
        assert_eq!(p.internal.result.unwrap().unwrap(), 1);
    }
}
