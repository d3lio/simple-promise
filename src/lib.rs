use std::cell::RefCell;
use std::rc::Rc;

type Resolve<T> = dyn FnOnce(T);
type Reject<E> = dyn FnOnce(E);

struct InternalPromise<T, E> {
    result: Option<Result<T, E>>,
}

pub struct Promise<T, E> {
    internal: Rc<RefCell<InternalPromise<T, E>>>,
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
        let internal = Rc::new(RefCell::new(InternalPromise::new()));
        let resolver = internal.clone();
        let rejecter = internal.clone();

        executor(
            Box::new(move |value| resolver.borrow_mut().resolve(Some(value))),
            Box::new(move |reason| rejecter.borrow_mut().reject(Some(reason))),
        );

        Self {
            internal,
        }
    }

    pub fn is_done(&self) -> bool {
        self.internal.borrow().is_done()
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

        assert!(p.internal.borrow().result.is_some());
        assert!(p.internal.borrow().result.unwrap().is_ok());
        assert_eq!(p.internal.borrow().result.unwrap().unwrap(), 1);
    }
}
